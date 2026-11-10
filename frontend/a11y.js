// Keyboard access to regions that scroll.
//
// A table wider than its column, an equation wider than its card and the unit
// panel taller than the window all scroll inside their own box. A reader who
// uses the keyboard cannot scroll a box that cannot take focus, so each of
// these boxes gets a tab stop and a name while, and only while, it overflows.
// The attributes are removed again when the content fits, so wide screens do
// not collect tab stops that do nothing.
//
// This file reads layout and writes attributes. It holds no model arithmetic.

const SELECTOR = ".table-scroll, .eq-body, .pfd-side";
const MARK = "data-scroll-region";
const HEADINGS = "h2, h3, h4, summary, figcaption";

function overflows(box) {
  const cs = getComputedStyle(box);
  const x = /(auto|scroll)/.test(cs.overflowX) && box.scrollWidth > box.clientWidth + 1;
  const y = /(auto|scroll)/.test(cs.overflowY) && box.scrollHeight > box.clientHeight + 1;
  return x || y;
}

function clean(text) {
  return String(text || "").replace(/\s+/g, " ").trim();
}

/** Text of an element with one space between its child nodes, since inline children such as a bold label carry none in the markup. */
function spoken(el) {
  return clean(Array.from(el.childNodes, (n) => n.textContent).join(" "));
}

/** The nearest caption or heading that says what the box holds. */
export function regionName(box) {
  if (box.matches(".pfd-side")) return "Unit panel and process tour";
  const cap = box.querySelector("caption");
  if (cap && clean(cap.textContent)) return clean(cap.textContent).slice(0, 90);
  const eq = box.closest(".eq");
  if (eq) {
    const head = eq.querySelector(".eq-head");
    const text = head ? spoken(head).slice(0, 80) : "";
    if (text) return /^Eq\./.test(text) ? text : `Equation ${text}`;
  }
  for (let n = box; n && n !== document.body; n = n.parentElement) {
    for (let s = n.previousElementSibling; s; s = s.previousElementSibling) {
      const h = s.matches(HEADINGS) ? s : s.querySelector(HEADINGS);
      if (h && clean(h.textContent)) return clean(h.textContent).slice(0, 90);
    }
    if (n.matches("details") && n.querySelector("summary")) return clean(n.querySelector("summary").textContent).slice(0, 90);
  }
  return "Table";
}

function mark(box) {
  if (box.hasAttribute(MARK)) return;
  box.setAttribute(MARK, "");
  box.setAttribute("tabindex", "0");
  box.setAttribute("aria-label", `${regionName(box)}, scrolls`);
  // An aside is a landmark already; a plain box needs a role to carry its name.
  if (!box.matches("aside")) box.setAttribute("role", "group");
}

function unmark(box) {
  if (!box.hasAttribute(MARK)) return;
  box.removeAttribute(MARK);
  box.removeAttribute("tabindex");
  box.removeAttribute("aria-label");
  if (box.getAttribute("role") === "group") box.removeAttribute("role");
}

/** Give every overflowing scroll box a tab stop and a name; take both away from the rest. */
export function syncScrollRegions(root = document) {
  for (const box of root.querySelectorAll(SELECTOR)) {
    if (overflows(box)) mark(box);
    else unmark(box);
  }
}

/** Keep the boxes in step with the page: content changes, route changes and resizes. */
export function watchScrollRegions() {
  let queued = false;
  const queue = () => {
    if (queued) return;
    queued = true;
    requestAnimationFrame(() => {
      queued = false;
      syncScrollRegions();
    });
  };
  new MutationObserver(queue).observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ["hidden"],
  });
  window.addEventListener("resize", queue);
  queue();
}
