// What the page tests (tests/page.rs) find on the page and measure there, as `novel.…` in the
// scripts they run. Every document of the page runs this first. Things are found by their role
// and their name, as a reader of the screen is told them; the roles and the names are worked
// out here as the browser does for what this page has (the browser's own, `computedRole`, takes
// a millisecond an element). Written after tests/support/page.js of video-desk.
window.novel = (() => {
  /** The elements that have a role without saying so. */
  const IMPLICIT = {
    button:
      "button, input[type=button], input[type=submit], input[type=reset], input[type=image]",
    link: "a[href], area[href]",
    textbox:
      "textarea, input:not([type]), input[type=text], input[type=email], input[type=tel], input[type=url]",
    searchbox: "input[type=search]",
    checkbox: "input[type=checkbox]",
    radio: "input[type=radio]",
    slider: "input[type=range]",
    spinbutton: "input[type=number]",
    combobox: "select:not([multiple]):not([size])",
    listbox: "select[multiple], select[size]",
    option: "option",
    list: "ul, ol",
    listitem: "ul > li, ol > li",
    dialog: "dialog",
    group: "fieldset, details, optgroup",
    img: "img:not([alt=''])",
    heading: "h1, h2, h3, h4, h5, h6",
    navigation: "nav",
    main: "main",
    status: "output",
    form: "form[aria-label], form[aria-labelledby]",
  };
  /** The role of `element`: the one it says, or the one it has without saying so. */
  function role(element) {
    const said = element.getAttribute("role")?.trim().split(/\s+/)[0];
    if (said) {
      return said;
    }
    return (
      Object.keys(IMPLICIT).find((one) => element.matches(IMPLICIT[one])) ?? ""
    );
  }
  /** Whether `element` is out of what a reader of the screen is told. */
  const hidden = (element) =>
    !element.checkVisibility({ visibilityProperty: true }) ||
    element.closest("[aria-hidden=true]") !== null;
  const flat = (text) => text.replace(/\s+/g, " ").trim();
  /** The roles whose name is what they hold. */
  const FROM_CONTENT = new Set([
    "button",
    "link",
    "tab",
    "menuitem",
    "menuitemradio",
    "menuitemcheckbox",
    "option",
    "radio",
    "checkbox",
    "switch",
    "heading",
    "cell",
    "columnheader",
    "rowheader",
    "tooltip",
    "treeitem",
  ]);
  /** The text `node` gives the name of what holds it. */
  function held(node) {
    if (node.nodeType === Node.TEXT_NODE) {
      return node.textContent;
    }
    if (node.nodeType !== Node.ELEMENT_NODE || hidden(node)) {
      return "";
    }
    const label = node.getAttribute("aria-label")?.trim();
    if (label) {
      return label;
    }
    if (node.matches("img")) {
      return node.getAttribute("alt") ?? "";
    }
    const inline = getComputedStyle(node).display.startsWith("inline");
    const inner = [...node.childNodes].map(held).join("");
    return inline ? inner : ` ${inner} `;
  }
  /** The name of `element`, as a reader of the screen is told it. */
  function name(element) {
    const by = element.getAttribute("aria-labelledby");
    if (by) {
      return flat(
        by
          .split(/\s+/)
          .map((id) => document.getElementById(id))
          .filter(Boolean)
          .map((one) => one.getAttribute("aria-label") ?? held(one))
          .join(" "),
      );
    }
    const label = element.getAttribute("aria-label")?.trim();
    if (label) {
      return flat(label);
    }
    if (element.labels?.length) {
      return flat([...element.labels].map(held).join(" "));
    }
    if (element.matches("fieldset")) {
      const legend = element.querySelector(":scope > legend");
      if (legend) {
        return flat(held(legend));
      }
    }
    if (element.matches("img")) {
      return flat(element.getAttribute("alt") ?? "");
    }
    if (FROM_CONTENT.has(role(element))) {
      const inner = flat([...element.childNodes].map(held).join(""));
      if (inner) {
        return inner;
      }
    }
    return flat(element.getAttribute("title") ?? "");
  }
  const named = (element, wanted) =>
    wanted === undefined ||
    (wanted instanceof RegExp
      ? wanted.test(name(element))
      : name(element) === wanted);
  /** Every element with `role` and `name` (a string, whole, or a RegExp) in `within`, that a reader of the screen is told of. */
  function all(role_, wanted, within = document.body) {
    if (!within) {
      return [];
    }
    const may = `[role~="${role_}"]${IMPLICIT[role_] ? `, ${IMPLICIT[role_]}` : ""}`;
    return [within, ...within.querySelectorAll(may)].filter(
      (element) =>
        element.matches(may) &&
        role(element) === role_ &&
        !hidden(element) &&
        named(element, wanted),
    );
  }
  /** The one element with `role` and `name` in `within`: null while there is none. */
  function one(role_, wanted, within) {
    const found = all(role_, wanted, within);
    if (found.length > 1) {
      throw new Error(`${found.length} elements are ${role_} ${wanted}`);
    }
    return found[0] ?? null;
  }
  /** The box of `element` in the window, as [x, y, width, height]. */
  function box(element) {
    const { x, y, width, height } = element.getBoundingClientRect();
    return [x, y, width, height];
  }

  /** A colour of CSS, as a computed style says it (`rgb(…)`). */
  function rgb(colour) {
    const probe = document.createElement("i");
    probe.style.color = colour;
    document.body.append(probe);
    const computed = getComputedStyle(probe).color;
    probe.remove();
    return computed;
  }

  return {
    role,
    name,
    all,
    one,
    box,
    rgb,
    /** Whether `element` is there (not null or undefined) and shows: it has a box, and is not hidden. */
    shows: (element) =>
      element != null &&
      element.checkVisibility({ visibilityProperty: true }) &&
      element.getBoundingClientRect().width > 0,
    /** Whether the page reaches beyond the right side of the window. */
    spills: () => {
      const root = document.documentElement;
      return root.scrollWidth > root.clientWidth + 1;
    },
    /**
     * The theme-color Chrome applies: the first meta[name=theme-color] in tree order whose
     * media matches (HTML spec; Blink Document::ThemeColor), as a computed style says it.
     */
    themeColor() {
      const meta = [...document.querySelectorAll('meta[name="theme-color"]')].find(
        (one) => !one.media || matchMedia(one.media).matches,
      );
      return meta ? rgb(meta.content) : null;
    },
    /**
     * Where the mouse presses `element`, once it is brought into the window: its middle, once
     * it is where it was a frame before and nothing else lies over it there; null while it
     * moves or something does.
     */
    async aim(element) {
      element.scrollIntoViewIfNeeded(true);
      const before = box(element).join();
      await new Promise((done) => requestAnimationFrame(done));
      const [x, y, width, height] = box(element);
      const at = [x + width / 2, y + height / 2];
      return before === [x, y, width, height].join() &&
        element.contains(document.elementFromPoint(...at))
        ? at
        : null;
    },
    /**
     * Rasterises the icon at `url` as a launcher would and measures, as fractions of the icon
     * size, the pixels outside the safe zone (radius 40%) that differ from the corner
     * background and the farthest artwork pixel from the centre.
     */
    async maskable(url) {
      const size = 512;
      const image = new Image();
      image.src = url;
      await image.decode();
      const canvas = document.createElement("canvas");
      canvas.width = canvas.height = size;
      const context = canvas.getContext("2d");
      context.drawImage(image, 0, 0, size, size);
      const pixels = context.getImageData(0, 0, size, size).data;
      const background = pixels.slice(0, 4);
      let outsideSafeZone = 0;
      let artworkRadius = 0;
      for (let y = 0; y < size; y++) {
        for (let x = 0; x < size; x++) {
          const i = (y * size + x) * 4;
          const diff = Math.max(...[0, 1, 2, 3].map((c) => Math.abs(pixels[i + c] - background[c])));
          const radius = Math.hypot(x + 0.5 - size / 2, y + 0.5 - size / 2) / size;
          if (radius > 0.4 && diff > 2) outsideSafeZone++;
          if (diff > 32) artworkRadius = Math.max(artworkRadius, radius);
        }
      }
      return { opaque: background[3] === 255, outsideSafeZone, artworkRadius };
    },
  };
})();
