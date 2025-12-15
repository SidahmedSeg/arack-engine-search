import { z as sanitize_props, F as rest_props, G as attributes, J as clsx, K as ensure_array_like, N as element, O as slot, P as bind_props, Q as spread_props, T as attr, x as attr_class } from "./index2.js";
import { a as ssr_context, e as escape_html } from "./context.js";
import { clsx as clsx$1 } from "clsx";
import DOMPurify from "dompurify";
import { twMerge } from "tailwind-merge";
import "@tiptap/starter-kit";
import "@tiptap/extension-link";
import "@tiptap/extension-placeholder";
import { j as fallback } from "./utils2.js";
import { Centrifuge } from "centrifuge";
import axios from "axios";
function html(value) {
  var html2 = String(value ?? "");
  var open = "<!---->";
  return open + html2 + "<!---->";
}
function onDestroy(fn) {
  /** @type {SSRContext} */
  ssr_context.r.on_destroy(fn);
}
/**
 * @license lucide-svelte v0.468.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const defaultAttributes = {
  xmlns: "http://www.w3.org/2000/svg",
  width: 24,
  height: 24,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  "stroke-width": 2,
  "stroke-linecap": "round",
  "stroke-linejoin": "round"
};
function Icon($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  const $$restProps = rest_props($$sanitized_props, [
    "name",
    "color",
    "size",
    "strokeWidth",
    "absoluteStrokeWidth",
    "iconNode"
  ]);
  $$renderer.component(($$renderer2) => {
    let name = fallback($$props["name"], void 0);
    let color = fallback($$props["color"], "currentColor");
    let size = fallback($$props["size"], 24);
    let strokeWidth = fallback($$props["strokeWidth"], 2);
    let absoluteStrokeWidth = fallback($$props["absoluteStrokeWidth"], false);
    let iconNode = fallback($$props["iconNode"], () => [], true);
    const mergeClasses = (...classes) => classes.filter((className, index, array) => {
      return Boolean(className) && array.indexOf(className) === index;
    }).join(" ");
    $$renderer2.push(`<svg${attributes(
      {
        ...defaultAttributes,
        ...$$restProps,
        width: size,
        height: size,
        stroke: color,
        "stroke-width": absoluteStrokeWidth ? Number(strokeWidth) * 24 / Number(size) : strokeWidth,
        class: clsx(mergeClasses("lucide-icon", "lucide", name ? `lucide-${name}` : "", $$sanitized_props.class))
      },
      void 0,
      void 0,
      void 0,
      3
    )}><!--[-->`);
    const each_array = ensure_array_like(iconNode);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let [tag, attrs] = each_array[$$index];
      element($$renderer2, tag, () => {
        $$renderer2.push(`${attributes({ ...attrs }, void 0, void 0, void 0, 3)}`);
      });
    }
    $$renderer2.push(`<!--]--><!--[-->`);
    slot($$renderer2, $$props, "default", {});
    $$renderer2.push(`<!--]--></svg>`);
    bind_props($$props, {
      name,
      color,
      size,
      strokeWidth,
      absoluteStrokeWidth,
      iconNode
    });
  });
}
function Archive($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    [
      "rect",
      { "width": "20", "height": "5", "x": "2", "y": "3", "rx": "1" }
    ],
    ["path", { "d": "M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8" }],
    ["path", { "d": "M10 12h4" }]
  ];
  Icon($$renderer, spread_props([
    { name: "archive" },
    $$sanitized_props,
    {
      /**
       * @component @name Archive
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cmVjdCB3aWR0aD0iMjAiIGhlaWdodD0iNSIgeD0iMiIgeT0iMyIgcng9IjEiIC8+CiAgPHBhdGggZD0iTTQgOHYxMWEyIDIgMCAwIDAgMiAyaDEyYTIgMiAwIDAgMCAyLTJWOCIgLz4KICA8cGF0aCBkPSJNMTAgMTJoNCIgLz4KPC9zdmc+Cg==) - https://lucide.dev/icons/archive
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Arrow_left($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "m12 19-7-7 7-7" }],
    ["path", { "d": "M19 12H5" }]
  ];
  Icon($$renderer, spread_props([
    { name: "arrow-left" },
    $$sanitized_props,
    {
      /**
       * @component @name ArrowLeft
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJtMTIgMTktNy03IDctNyIgLz4KICA8cGF0aCBkPSJNMTkgMTJINSIgLz4KPC9zdmc+Cg==) - https://lucide.dev/icons/arrow-left
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Bold($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    [
      "path",
      {
        "d": "M6 12h9a4 4 0 0 1 0 8H7a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h7a4 4 0 0 1 0 8"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "bold" },
    $$sanitized_props,
    {
      /**
       * @component @name Bold
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNNiAxMmg5YTQgNCAwIDAgMSAwIDhIN2ExIDEgMCAwIDEtMS0xVjVhMSAxIDAgMCAxIDEtMWg3YTQgNCAwIDAgMSAwIDgiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/bold
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Chevron_left($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [["path", { "d": "m15 18-6-6 6-6" }]];
  Icon($$renderer, spread_props([
    { name: "chevron-left" },
    $$sanitized_props,
    {
      /**
       * @component @name ChevronLeft
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJtMTUgMTgtNi02IDYtNiIgLz4KPC9zdmc+Cg==) - https://lucide.dev/icons/chevron-left
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Chevron_right($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [["path", { "d": "m9 18 6-6-6-6" }]];
  Icon($$renderer, spread_props([
    { name: "chevron-right" },
    $$sanitized_props,
    {
      /**
       * @component @name ChevronRight
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJtOSAxOCA2LTYtNi02IiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/chevron-right
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Circle_alert($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["circle", { "cx": "12", "cy": "12", "r": "10" }],
    ["line", { "x1": "12", "x2": "12", "y1": "8", "y2": "12" }],
    [
      "line",
      { "x1": "12", "x2": "12.01", "y1": "16", "y2": "16" }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "circle-alert" },
    $$sanitized_props,
    {
      /**
       * @component @name CircleAlert
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8Y2lyY2xlIGN4PSIxMiIgY3k9IjEyIiByPSIxMCIgLz4KICA8bGluZSB4MT0iMTIiIHgyPSIxMiIgeTE9IjgiIHkyPSIxMiIgLz4KICA8bGluZSB4MT0iMTIiIHgyPSIxMi4wMSIgeTE9IjE2IiB5Mj0iMTYiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/circle-alert
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Code($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["polyline", { "points": "16 18 22 12 16 6" }],
    ["polyline", { "points": "8 6 2 12 8 18" }]
  ];
  Icon($$renderer, spread_props([
    { name: "code" },
    $$sanitized_props,
    {
      /**
       * @component @name Code
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cG9seWxpbmUgcG9pbnRzPSIxNiAxOCAyMiAxMiAxNiA2IiAvPgogIDxwb2x5bGluZSBwb2ludHM9IjggNiAyIDEyIDggMTgiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/code
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Download($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }],
    ["polyline", { "points": "7 10 12 15 17 10" }],
    ["line", { "x1": "12", "x2": "12", "y1": "15", "y2": "3" }]
  ];
  Icon($$renderer, spread_props([
    { name: "download" },
    $$sanitized_props,
    {
      /**
       * @component @name Download
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMjEgMTV2NGEyIDIgMCAwIDEtMiAySDVhMiAyIDAgMCAxLTItMnYtNCIgLz4KICA8cG9seWxpbmUgcG9pbnRzPSI3IDEwIDEyIDE1IDE3IDEwIiAvPgogIDxsaW5lIHgxPSIxMiIgeDI9IjEyIiB5MT0iMTUiIHkyPSIzIiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/download
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Ellipsis_vertical($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["circle", { "cx": "12", "cy": "12", "r": "1" }],
    ["circle", { "cx": "12", "cy": "5", "r": "1" }],
    ["circle", { "cx": "12", "cy": "19", "r": "1" }]
  ];
  Icon($$renderer, spread_props([
    { name: "ellipsis-vertical" },
    $$sanitized_props,
    {
      /**
       * @component @name EllipsisVertical
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8Y2lyY2xlIGN4PSIxMiIgY3k9IjEyIiByPSIxIiAvPgogIDxjaXJjbGUgY3g9IjEyIiBjeT0iNSIgcj0iMSIgLz4KICA8Y2lyY2xlIGN4PSIxMiIgY3k9IjE5IiByPSIxIiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/ellipsis-vertical
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function File_text($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    [
      "path",
      {
        "d": "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"
      }
    ],
    ["path", { "d": "M14 2v4a2 2 0 0 0 2 2h4" }],
    ["path", { "d": "M10 9H8" }],
    ["path", { "d": "M16 13H8" }],
    ["path", { "d": "M16 17H8" }]
  ];
  Icon($$renderer, spread_props([
    { name: "file-text" },
    $$sanitized_props,
    {
      /**
       * @component @name FileText
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTUgMkg2YTIgMiAwIDAgMC0yIDJ2MTZhMiAyIDAgMCAwIDIgMmgxMmEyIDIgMCAwIDAgMi0yVjdaIiAvPgogIDxwYXRoIGQ9Ik0xNCAydjRhMiAyIDAgMCAwIDIgMmg0IiAvPgogIDxwYXRoIGQ9Ik0xMCA5SDgiIC8+CiAgPHBhdGggZD0iTTE2IDEzSDgiIC8+CiAgPHBhdGggZD0iTTE2IDE3SDgiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/file-text
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Folder($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    [
      "path",
      {
        "d": "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "folder" },
    $$sanitized_props,
    {
      /**
       * @component @name Folder
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMjAgMjBhMiAyIDAgMCAwIDItMlY4YTIgMiAwIDAgMC0yLTJoLTcuOWEyIDIgMCAwIDEtMS42OS0uOUw5LjYgMy45QTIgMiAwIDAgMCA3LjkzIDNINGEyIDIgMCAwIDAtMiAydjEzYTIgMiAwIDAgMCAyIDJaIiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/folder
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Forward($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["polyline", { "points": "15 17 20 12 15 7" }],
    ["path", { "d": "M4 18v-2a4 4 0 0 1 4-4h12" }]
  ];
  Icon($$renderer, spread_props([
    { name: "forward" },
    $$sanitized_props,
    {
      /**
       * @component @name Forward
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cG9seWxpbmUgcG9pbnRzPSIxNSAxNyAyMCAxMiAxNSA3IiAvPgogIDxwYXRoIGQ9Ik00IDE4di0yYTQgNCAwIDAgMSA0LTRoMTIiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/forward
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Inbox($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    [
      "polyline",
      { "points": "22 12 16 12 14 15 10 15 8 12 2 12" }
    ],
    [
      "path",
      {
        "d": "M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "inbox" },
    $$sanitized_props,
    {
      /**
       * @component @name Inbox
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cG9seWxpbmUgcG9pbnRzPSIyMiAxMiAxNiAxMiAxNCAxNSAxMCAxNSA4IDEyIDIgMTIiIC8+CiAgPHBhdGggZD0iTTUuNDUgNS4xMSAyIDEydjZhMiAyIDAgMCAwIDIgMmgxNmEyIDIgMCAwIDAgMi0ydi02bC0zLjQ1LTYuODlBMiAyIDAgMCAwIDE2Ljc2IDRINy4yNGEyIDIgMCAwIDAtMS43OSAxLjExeiIgLz4KPC9zdmc+Cg==) - https://lucide.dev/icons/inbox
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Italic($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["line", { "x1": "19", "x2": "10", "y1": "4", "y2": "4" }],
    ["line", { "x1": "14", "x2": "5", "y1": "20", "y2": "20" }],
    ["line", { "x1": "15", "x2": "9", "y1": "4", "y2": "20" }]
  ];
  Icon($$renderer, spread_props([
    { name: "italic" },
    $$sanitized_props,
    {
      /**
       * @component @name Italic
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8bGluZSB4MT0iMTkiIHgyPSIxMCIgeTE9IjQiIHkyPSI0IiAvPgogIDxsaW5lIHgxPSIxNCIgeDI9IjUiIHkxPSIyMCIgeTI9IjIwIiAvPgogIDxsaW5lIHgxPSIxNSIgeDI9IjkiIHkxPSI0IiB5Mj0iMjAiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/italic
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Link_2($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M9 17H7A5 5 0 0 1 7 7h2" }],
    ["path", { "d": "M15 7h2a5 5 0 1 1 0 10h-2" }],
    ["line", { "x1": "8", "x2": "16", "y1": "12", "y2": "12" }]
  ];
  Icon($$renderer, spread_props([
    { name: "link-2" },
    $$sanitized_props,
    {
      /**
       * @component @name Link2
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNOSAxN0g3QTUgNSAwIDAgMSA3IDdoMiIgLz4KICA8cGF0aCBkPSJNMTUgN2gyYTUgNSAwIDEgMSAwIDEwaC0yIiAvPgogIDxsaW5lIHgxPSI4IiB4Mj0iMTYiIHkxPSIxMiIgeTI9IjEyIiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/link-2
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function List_ordered($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M10 12h11" }],
    ["path", { "d": "M10 18h11" }],
    ["path", { "d": "M10 6h11" }],
    ["path", { "d": "M4 10h2" }],
    ["path", { "d": "M4 6h1v4" }],
    ["path", { "d": "M6 18H4c0-1 2-2 2-3s-1-1.5-2-1" }]
  ];
  Icon($$renderer, spread_props([
    { name: "list-ordered" },
    $$sanitized_props,
    {
      /**
       * @component @name ListOrdered
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTAgMTJoMTEiIC8+CiAgPHBhdGggZD0iTTEwIDE4aDExIiAvPgogIDxwYXRoIGQ9Ik0xMCA2aDExIiAvPgogIDxwYXRoIGQ9Ik00IDEwaDIiIC8+CiAgPHBhdGggZD0iTTQgNmgxdjQiIC8+CiAgPHBhdGggZD0iTTYgMThINGMwLTEgMi0yIDItM3MtMS0xLjUtMi0xIiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/list-ordered
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function List($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M3 12h.01" }],
    ["path", { "d": "M3 18h.01" }],
    ["path", { "d": "M3 6h.01" }],
    ["path", { "d": "M8 12h13" }],
    ["path", { "d": "M8 18h13" }],
    ["path", { "d": "M8 6h13" }]
  ];
  Icon($$renderer, spread_props([
    { name: "list" },
    $$sanitized_props,
    {
      /**
       * @component @name List
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMyAxMmguMDEiIC8+CiAgPHBhdGggZD0iTTMgMThoLjAxIiAvPgogIDxwYXRoIGQ9Ik0zIDZoLjAxIiAvPgogIDxwYXRoIGQ9Ik04IDEyaDEzIiAvPgogIDxwYXRoIGQ9Ik04IDE4aDEzIiAvPgogIDxwYXRoIGQ9Ik04IDZoMTMiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/list
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Maximize_2($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["polyline", { "points": "15 3 21 3 21 9" }],
    ["polyline", { "points": "9 21 3 21 3 15" }],
    ["line", { "x1": "21", "x2": "14", "y1": "3", "y2": "10" }],
    ["line", { "x1": "3", "x2": "10", "y1": "21", "y2": "14" }]
  ];
  Icon($$renderer, spread_props([
    { name: "maximize-2" },
    $$sanitized_props,
    {
      /**
       * @component @name Maximize2
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cG9seWxpbmUgcG9pbnRzPSIxNSAzIDIxIDMgMjEgOSIgLz4KICA8cG9seWxpbmUgcG9pbnRzPSI5IDIxIDMgMjEgMyAxNSIgLz4KICA8bGluZSB4MT0iMjEiIHgyPSIxNCIgeTE9IjMiIHkyPSIxMCIgLz4KICA8bGluZSB4MT0iMyIgeDI9IjEwIiB5MT0iMjEiIHkyPSIxNCIgLz4KPC9zdmc+Cg==) - https://lucide.dev/icons/maximize-2
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Moon($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [["path", { "d": "M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" }]];
  Icon($$renderer, spread_props([
    { name: "moon" },
    $$sanitized_props,
    {
      /**
       * @component @name Moon
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTIgM2E2IDYgMCAwIDAgOSA5IDkgOSAwIDEgMS05LTlaIiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/moon
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Paperclip($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M13.234 20.252 21 12.3" }],
    [
      "path",
      {
        "d": "m16 6-8.414 8.586a2 2 0 0 0 0 2.828 2 2 0 0 0 2.828 0l8.414-8.586a4 4 0 0 0 0-5.656 4 4 0 0 0-5.656 0l-8.415 8.585a6 6 0 1 0 8.486 8.486"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "paperclip" },
    $$sanitized_props,
    {
      /**
       * @component @name Paperclip
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTMuMjM0IDIwLjI1MiAyMSAxMi4zIiAvPgogIDxwYXRoIGQ9Im0xNiA2LTguNDE0IDguNTg2YTIgMiAwIDAgMCAwIDIuODI4IDIgMiAwIDAgMCAyLjgyOCAwbDguNDE0LTguNTg2YTQgNCAwIDAgMCAwLTUuNjU2IDQgNCAwIDAgMC01LjY1NiAwbC04LjQxNSA4LjU4NWE2IDYgMCAxIDAgOC40ODYgOC40ODYiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/paperclip
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Plus($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [["path", { "d": "M5 12h14" }], ["path", { "d": "M12 5v14" }]];
  Icon($$renderer, spread_props([
    { name: "plus" },
    $$sanitized_props,
    {
      /**
       * @component @name Plus
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNNSAxMmgxNCIgLz4KICA8cGF0aCBkPSJNMTIgNXYxNCIgLz4KPC9zdmc+Cg==) - https://lucide.dev/icons/plus
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Reply_all($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["polyline", { "points": "7 17 2 12 7 7" }],
    ["polyline", { "points": "12 17 7 12 12 7" }],
    ["path", { "d": "M22 18v-2a4 4 0 0 0-4-4H7" }]
  ];
  Icon($$renderer, spread_props([
    { name: "reply-all" },
    $$sanitized_props,
    {
      /**
       * @component @name ReplyAll
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cG9seWxpbmUgcG9pbnRzPSI3IDE3IDIgMTIgNyA3IiAvPgogIDxwb2x5bGluZSBwb2ludHM9IjEyIDE3IDcgMTIgMTIgNyIgLz4KICA8cGF0aCBkPSJNMjIgMTh2LTJhNCA0IDAgMCAwLTQtNEg3IiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/reply-all
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Reply($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["polyline", { "points": "9 17 4 12 9 7" }],
    ["path", { "d": "M20 18v-2a4 4 0 0 0-4-4H4" }]
  ];
  Icon($$renderer, spread_props([
    { name: "reply" },
    $$sanitized_props,
    {
      /**
       * @component @name Reply
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cG9seWxpbmUgcG9pbnRzPSI5IDE3IDQgMTIgOSA3IiAvPgogIDxwYXRoIGQ9Ik0yMCAxOHYtMmE0IDQgMCAwIDAtNC00SDQiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/reply
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Search($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["circle", { "cx": "11", "cy": "11", "r": "8" }],
    ["path", { "d": "m21 21-4.3-4.3" }]
  ];
  Icon($$renderer, spread_props([
    { name: "search" },
    $$sanitized_props,
    {
      /**
       * @component @name Search
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8Y2lyY2xlIGN4PSIxMSIgY3k9IjExIiByPSI4IiAvPgogIDxwYXRoIGQ9Im0yMSAyMS00LjMtNC4zIiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/search
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Send($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    [
      "path",
      {
        "d": "M14.536 21.686a.5.5 0 0 0 .937-.024l6.5-19a.496.496 0 0 0-.635-.635l-19 6.5a.5.5 0 0 0-.024.937l7.93 3.18a2 2 0 0 1 1.112 1.11z"
      }
    ],
    ["path", { "d": "m21.854 2.147-10.94 10.939" }]
  ];
  Icon($$renderer, spread_props([
    { name: "send" },
    $$sanitized_props,
    {
      /**
       * @component @name Send
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTQuNTM2IDIxLjY4NmEuNS41IDAgMCAwIC45MzctLjAyNGw2LjUtMTlhLjQ5Ni40OTYgMCAwIDAtLjYzNS0uNjM1bC0xOSA2LjVhLjUuNSAwIDAgMC0uMDI0LjkzN2w3LjkzIDMuMThhMiAyIDAgMCAxIDEuMTEyIDEuMTF6IiAvPgogIDxwYXRoIGQ9Im0yMS44NTQgMi4xNDctMTAuOTQgMTAuOTM5IiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/send
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Settings($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    [
      "path",
      {
        "d": "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
      }
    ],
    ["circle", { "cx": "12", "cy": "12", "r": "3" }]
  ];
  Icon($$renderer, spread_props([
    { name: "settings" },
    $$sanitized_props,
    {
      /**
       * @component @name Settings
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTIuMjIgMmgtLjQ0YTIgMiAwIDAgMC0yIDJ2LjE4YTIgMiAwIDAgMS0xIDEuNzNsLS40My4yNWEyIDIgMCAwIDEtMiAwbC0uMTUtLjA4YTIgMiAwIDAgMC0yLjczLjczbC0uMjIuMzhhMiAyIDAgMCAwIC43MyAyLjczbC4xNS4xYTIgMiAwIDAgMSAxIDEuNzJ2LjUxYTIgMiAwIDAgMS0xIDEuNzRsLS4xNS4wOWEyIDIgMCAwIDAtLjczIDIuNzNsLjIyLjM4YTIgMiAwIDAgMCAyLjczLjczbC4xNS0uMDhhMiAyIDAgMCAxIDIgMGwuNDMuMjVhMiAyIDAgMCAxIDEgMS43M1YyMGEyIDIgMCAwIDAgMiAyaC40NGEyIDIgMCAwIDAgMi0ydi0uMThhMiAyIDAgMCAxIDEtMS43M2wuNDMtLjI1YTIgMiAwIDAgMSAyIDBsLjE1LjA4YTIgMiAwIDAgMCAyLjczLS43M2wuMjItLjM5YTIgMiAwIDAgMC0uNzMtMi43M2wtLjE1LS4wOGEyIDIgMCAwIDEtMS0xLjc0di0uNWEyIDIgMCAwIDEgMS0xLjc0bC4xNS0uMDlhMiAyIDAgMCAwIC43My0yLjczbC0uMjItLjM4YTIgMiAwIDAgMC0yLjczLS43M2wtLjE1LjA4YTIgMiAwIDAgMS0yIDBsLS40My0uMjVhMiAyIDAgMCAxLTEtMS43M1Y0YTIgMiAwIDAgMC0yLTJ6IiAvPgogIDxjaXJjbGUgY3g9IjEyIiBjeT0iMTIiIHI9IjMiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/settings
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Square_pen($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    [
      "path",
      {
        "d": "M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
      }
    ],
    [
      "path",
      {
        "d": "M18.375 2.625a1 1 0 0 1 3 3l-9.013 9.014a2 2 0 0 1-.853.505l-2.873.84a.5.5 0 0 1-.62-.62l.84-2.873a2 2 0 0 1 .506-.852z"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "square-pen" },
    $$sanitized_props,
    {
      /**
       * @component @name SquarePen
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTIgM0g1YTIgMiAwIDAgMC0yIDJ2MTRhMiAyIDAgMCAwIDIgMmgxNGEyIDIgMCAwIDAgMi0ydi03IiAvPgogIDxwYXRoIGQ9Ik0xOC4zNzUgMi42MjVhMSAxIDAgMCAxIDMgM2wtOS4wMTMgOS4wMTRhMiAyIDAgMCAxLS44NTMuNTA1bC0yLjg3My44NGEuNS41IDAgMCAxLS42Mi0uNjJsLjg0LTIuODczYTIgMiAwIDAgMSAuNTA2LS44NTJ6IiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/square-pen
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Star($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    [
      "path",
      {
        "d": "M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "star" },
    $$sanitized_props,
    {
      /**
       * @component @name Star
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTEuNTI1IDIuMjk1YS41My41MyAwIDAgMSAuOTUgMGwyLjMxIDQuNjc5YTIuMTIzIDIuMTIzIDAgMCAwIDEuNTk1IDEuMTZsNS4xNjYuNzU2YS41My41MyAwIDAgMSAuMjk0LjkwNGwtMy43MzYgMy42MzhhMi4xMjMgMi4xMjMgMCAwIDAtLjYxMSAxLjg3OGwuODgyIDUuMTRhLjUzLjUzIDAgMCAxLS43NzEuNTZsLTQuNjE4LTIuNDI4YTIuMTIyIDIuMTIyIDAgMCAwLTEuOTczIDBMNi4zOTYgMjEuMDFhLjUzLjUzIDAgMCAxLS43Ny0uNTZsLjg4MS01LjEzOWEyLjEyMiAyLjEyMiAwIDAgMC0uNjExLTEuODc5TDIuMTYgOS43OTVhLjUzLjUzIDAgMCAxIC4yOTQtLjkwNmw1LjE2NS0uNzU1YTIuMTIyIDIuMTIyIDAgMCAwIDEuNTk3LTEuMTZ6IiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/star
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Sun($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["circle", { "cx": "12", "cy": "12", "r": "4" }],
    ["path", { "d": "M12 2v2" }],
    ["path", { "d": "M12 20v2" }],
    ["path", { "d": "m4.93 4.93 1.41 1.41" }],
    ["path", { "d": "m17.66 17.66 1.41 1.41" }],
    ["path", { "d": "M2 12h2" }],
    ["path", { "d": "M20 12h2" }],
    ["path", { "d": "m6.34 17.66-1.41 1.41" }],
    ["path", { "d": "m19.07 4.93-1.41 1.41" }]
  ];
  Icon($$renderer, spread_props([
    { name: "sun" },
    $$sanitized_props,
    {
      /**
       * @component @name Sun
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8Y2lyY2xlIGN4PSIxMiIgY3k9IjEyIiByPSI0IiAvPgogIDxwYXRoIGQ9Ik0xMiAydjIiIC8+CiAgPHBhdGggZD0iTTEyIDIwdjIiIC8+CiAgPHBhdGggZD0ibTQuOTMgNC45MyAxLjQxIDEuNDEiIC8+CiAgPHBhdGggZD0ibTE3LjY2IDE3LjY2IDEuNDEgMS40MSIgLz4KICA8cGF0aCBkPSJNMiAxMmgyIiAvPgogIDxwYXRoIGQ9Ik0yMCAxMmgyIiAvPgogIDxwYXRoIGQ9Im02LjM0IDE3LjY2LTEuNDEgMS40MSIgLz4KICA8cGF0aCBkPSJtMTkuMDcgNC45My0xLjQxIDEuNDEiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/sun
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Trash_2($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M3 6h18" }],
    ["path", { "d": "M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" }],
    ["path", { "d": "M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" }],
    ["line", { "x1": "10", "x2": "10", "y1": "11", "y2": "17" }],
    ["line", { "x1": "14", "x2": "14", "y1": "11", "y2": "17" }]
  ];
  Icon($$renderer, spread_props([
    { name: "trash-2" },
    $$sanitized_props,
    {
      /**
       * @component @name Trash2
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMyA2aDE4IiAvPgogIDxwYXRoIGQ9Ik0xOSA2djE0YzAgMS0xIDItMiAySDdjLTEgMC0yLTEtMi0yVjYiIC8+CiAgPHBhdGggZD0iTTggNlY0YzAtMSAxLTIgMi0yaDRjMSAwIDIgMSAyIDJ2MiIgLz4KICA8bGluZSB4MT0iMTAiIHgyPSIxMCIgeTE9IjExIiB5Mj0iMTciIC8+CiAgPGxpbmUgeDE9IjE0IiB4Mj0iMTQiIHkxPSIxMSIgeTI9IjE3IiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/trash-2
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Wifi_off($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M12 20h.01" }],
    ["path", { "d": "M8.5 16.429a5 5 0 0 1 7 0" }],
    ["path", { "d": "M5 12.859a10 10 0 0 1 5.17-2.69" }],
    ["path", { "d": "M19 12.859a10 10 0 0 0-2.007-1.523" }],
    ["path", { "d": "M2 8.82a15 15 0 0 1 4.177-2.643" }],
    ["path", { "d": "M22 8.82a15 15 0 0 0-11.288-3.764" }],
    ["path", { "d": "m2 2 20 20" }]
  ];
  Icon($$renderer, spread_props([
    { name: "wifi-off" },
    $$sanitized_props,
    {
      /**
       * @component @name WifiOff
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTIgMjBoLjAxIiAvPgogIDxwYXRoIGQ9Ik04LjUgMTYuNDI5YTUgNSAwIDAgMSA3IDAiIC8+CiAgPHBhdGggZD0iTTUgMTIuODU5YTEwIDEwIDAgMCAxIDUuMTctMi42OSIgLz4KICA8cGF0aCBkPSJNMTkgMTIuODU5YTEwIDEwIDAgMCAwLTIuMDA3LTEuNTIzIiAvPgogIDxwYXRoIGQ9Ik0yIDguODJhMTUgMTUgMCAwIDEgNC4xNzctMi42NDMiIC8+CiAgPHBhdGggZD0iTTIyIDguODJhMTUgMTUgMCAwIDAtMTEuMjg4LTMuNzY0IiAvPgogIDxwYXRoIGQ9Im0yIDIgMjAgMjAiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/wifi-off
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function Wifi($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M12 20h.01" }],
    ["path", { "d": "M2 8.82a15 15 0 0 1 20 0" }],
    ["path", { "d": "M5 12.859a10 10 0 0 1 14 0" }],
    ["path", { "d": "M8.5 16.429a5 5 0 0 1 7 0" }]
  ];
  Icon($$renderer, spread_props([
    { name: "wifi" },
    $$sanitized_props,
    {
      /**
       * @component @name Wifi
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTIgMjBoLjAxIiAvPgogIDxwYXRoIGQ9Ik0yIDguODJhMTUgMTUgMCAwIDEgMjAgMCIgLz4KICA8cGF0aCBkPSJNNSAxMi44NTlhMTAgMTAgMCAwIDEgMTQgMCIgLz4KICA8cGF0aCBkPSJNOC41IDE2LjQyOWE1IDUgMCAwIDEgNyAwIiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/wifi
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function X($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.468.0 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M18 6 6 18" }],
    ["path", { "d": "m6 6 12 12" }]
  ];
  Icon($$renderer, spread_props([
    { name: "x" },
    $$sanitized_props,
    {
      /**
       * @component @name X
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTggNiA2IDE4IiAvPgogIDxwYXRoIGQ9Im02IDYgMTIgMTIiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/x
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function cn(...inputs) {
  return twMerge(clsx$1(inputs));
}
function formatTimestamp(date) {
  const d = typeof date === "string" ? new Date(date) : date;
  const now = /* @__PURE__ */ new Date();
  const diff = now.getTime() - d.getTime();
  const diffDays = Math.floor(diff / (1e3 * 60 * 60 * 24));
  if (diffDays === 0) {
    return d.toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit", hour12: true });
  }
  if (diffDays === 1) {
    return "Yesterday";
  }
  if (diffDays < 7) {
    return d.toLocaleDateString("en-US", { weekday: "short" });
  }
  if (d.getFullYear() === now.getFullYear()) {
    return d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
  }
  return d.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
}
function getGravatarUrl(email, size = 40) {
  const hash = email.trim().toLowerCase();
  return `https://www.gravatar.com/avatar/${hash}?s=${size}&d=identicon`;
}
function getInitials(name) {
  const parts = name.trim().split(" ");
  if (parts.length >= 2) {
    return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
  }
  return name.substring(0, 2).toUpperCase();
}
function truncate(text, maxLength) {
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength) + "...";
}
function Button($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let {
      variant = "default",
      size = "md",
      class: className,
      disabled = false,
      type = "button",
      onclick,
      children
    } = $$props;
    const variantStyles = {
      default: "bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700",
      primary: "bg-primary-600 text-white border border-primary-600 hover:bg-primary-700 dark:bg-primary-500 dark:hover:bg-primary-600",
      secondary: "bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 border border-transparent hover:bg-gray-200 dark:hover:bg-gray-600",
      ghost: "bg-transparent text-gray-700 dark:text-gray-200 border border-transparent hover:bg-gray-100 dark:hover:bg-gray-800",
      danger: "bg-red-600 text-white border border-red-600 hover:bg-red-700 dark:bg-red-500 dark:hover:bg-red-600"
    };
    const sizeStyles = {
      sm: "h-8 px-3 text-sm",
      md: "h-9 px-4 text-sm",
      lg: "h-10 px-6 text-base",
      icon: "h-9 w-9 p-0"
    };
    $$renderer2.push(`<button${attr("type", type)}${attr("disabled", disabled, true)}${attr_class(clsx(cn("inline-flex items-center justify-center gap-2 rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50", variantStyles[variant], sizeStyles[size], className)))}>`);
    children?.($$renderer2);
    $$renderer2.push(`<!----></button>`);
  });
}
function MailboxList($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { mailboxes, currentMailbox, onMailboxSelect, onCompose } = $$props;
    const folderIcons = {
      inbox: Inbox,
      sent: Send,
      drafts: File_text,
      trash: Trash_2,
      junk: Circle_alert
    };
    function getIcon(role) {
      if (role && folderIcons[role]) {
        return folderIcons[role];
      }
      return Folder;
    }
    $$renderer2.push(`<div class="h-full w-64" style="background-color: #F8FAFD;"><div class="overflow-y-auto custom-scrollbar h-full py-4"><div class="px-3 mb-4">`);
    Button($$renderer2, {
      variant: "primary",
      size: "lg",
      onclick: onCompose,
      class: "w-full shadow-md",
      children: ($$renderer3) => {
        Square_pen($$renderer3, { class: "h-5 w-5" });
        $$renderer3.push(`<!----> <span>Compose</span>`);
      }
    });
    $$renderer2.push(`<!----></div> <div class="px-3 space-y-1"><!--[-->`);
    const each_array = ensure_array_like(mailboxes);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let mailbox = each_array[$$index];
      const Icon2 = getIcon(mailbox.role);
      $$renderer2.push(`<button${attr_class(clsx(cn("w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors", currentMailbox === mailbox.id ? "bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300" : "text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800")))}><!---->`);
      Icon2($$renderer2, { class: "h-5 w-5 flex-shrink-0" });
      $$renderer2.push(`<!----> <span class="flex-1 text-left">${escape_html(mailbox.name)}</span> `);
      if (mailbox.unread_emails > 0) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<span class="flex-shrink-0 px-2 py-0.5 text-xs font-semibold rounded-full bg-primary-600 text-white">${escape_html(mailbox.unread_emails)}</span>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></button>`);
    }
    $$renderer2.push(`<!--]--></div> <div class="mt-6 px-2"><div class="px-3 py-2 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase">Labels</div> <button class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">`);
    Plus($$renderer2, { class: "h-4 w-4" });
    $$renderer2.push(`<!----> <span>Create label</span></button></div></div></div>`);
  });
}
function MessageList($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { messages, selectedMessage, onMessageSelect, loading = false } = $$props;
    let selectedIds = /* @__PURE__ */ new Set();
    let selectAll = false;
    let currentPage = 1;
    let itemsPerPage = 50;
    $$renderer2.push(`<div class="h-full flex flex-col"><div class="flex-shrink-0 border-b border-gray-200 dark:border-gray-700"><div class="flex items-center px-4 pt-3"><label class="flex items-center cursor-pointer"><input type="checkbox"${attr(
      "checked",
      // Reset selection when messages change
      selectAll,
      true
    )} class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 focus:ring-2"/></label> <div class="flex-1 flex items-center ml-6"><button class="px-4 pb-3 text-sm font-medium text-primary-600 border-b-2 border-primary-600">Primary</button></div> <div class="flex items-center gap-2"><span class="text-xs text-gray-500 dark:text-gray-400">1-${escape_html(Math.min(messages.length, itemsPerPage))} of ${escape_html(messages.length)}</span> <button${attr("disabled", currentPage === 1, true)} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded disabled:opacity-50 disabled:cursor-not-allowed">`);
    Chevron_left($$renderer2, { class: "h-4 w-4" });
    $$renderer2.push(`<!----></button> <button${attr("disabled", currentPage >= Math.ceil(messages.length / itemsPerPage), true)} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded disabled:opacity-50 disabled:cursor-not-allowed">`);
    Chevron_right($$renderer2, { class: "h-4 w-4" });
    $$renderer2.push(`<!----></button></div></div></div> <div class="flex-1 overflow-y-auto custom-scrollbar">`);
    if (loading) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="flex items-center justify-center h-32"><div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
      if (messages.length === 0) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="flex flex-col items-center justify-center h-32 text-gray-500 dark:text-gray-400"><p class="text-sm">No messages</p></div>`);
      } else {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`<div><!--[-->`);
        const each_array = ensure_array_like(messages);
        for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
          let message = each_array[$$index];
          $$renderer2.push(`<button${attr_class(clsx(cn("w-full px-4 py-2.5 flex items-center gap-3 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors text-left group", selectedMessage?.id === message.id ? "bg-blue-50 dark:bg-blue-900/20" : "", !message.is_read ? "bg-blue-50/30 dark:bg-blue-900/10" : "")))}><div class="flex-shrink-0" role="presentation"><input type="checkbox"${attr("checked", selectedIds.has(message.id), true)} class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 focus:ring-2"/></div> <div class="flex-shrink-0"><div role="button" tabindex="0" class="p-0.5 hover:bg-gray-200 dark:hover:bg-gray-600 rounded cursor-pointer">`);
          Star($$renderer2, {
            class: message.is_starred ? "h-4 w-4 fill-yellow-400 text-yellow-400" : "h-4 w-4 text-gray-300 hover:text-yellow-400"
          });
          $$renderer2.push(`<!----></div></div> <div class="w-40 flex-shrink-0"><div${attr_class(clsx(cn("text-sm truncate", !message.is_read ? "font-semibold text-gray-900 dark:text-gray-100" : "font-normal text-gray-700 dark:text-gray-300")))}>${escape_html(message.from.name || message.from.email)}</div></div> <div class="flex-1 min-w-0"><div class="flex items-center gap-2"><span${attr_class(clsx(cn("text-sm truncate", !message.is_read ? "font-semibold text-gray-900 dark:text-gray-100" : "font-normal text-gray-700 dark:text-gray-300")))}>${escape_html(message.subject || "(no subject)")}</span> <span class="text-sm text-gray-500 dark:text-gray-400 truncate">— ${escape_html(truncate(message.preview, 60))}</span></div></div> <div class="flex-shrink-0 flex items-center gap-3">`);
          if (message.has_attachments) {
            $$renderer2.push("<!--[-->");
            Paperclip($$renderer2, { class: "h-4 w-4 text-gray-400" });
          } else {
            $$renderer2.push("<!--[!-->");
          }
          $$renderer2.push(`<!--]--> <span class="text-xs text-gray-500 dark:text-gray-400 w-16 text-right">${escape_html(formatTimestamp(message.received_at))}</span></div></button>`);
        }
        $$renderer2.push(`<!--]--></div>`);
      }
      $$renderer2.push(`<!--]-->`);
    }
    $$renderer2.push(`<!--]--></div></div>`);
  });
}
function Avatar($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { email, name = "", size = "md", class: className } = $$props;
    const sizeClasses = {
      sm: "h-8 w-8 text-xs",
      md: "h-10 w-10 text-sm",
      lg: "h-12 w-12 text-base"
    };
    const sizePixels = { sm: 32, md: 40, lg: 48 };
    const gravatarUrl = getGravatarUrl(email, sizePixels[size]);
    name ? getInitials(name) : email.substring(0, 2).toUpperCase();
    $$renderer2.push(`<div${attr_class(clsx(cn("relative inline-flex items-center justify-center rounded-full overflow-hidden", sizeClasses[size], className)))}>`);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<img${attr("src", gravatarUrl)}${attr("alt", name || email)} class="h-full w-full object-cover" onerror="this.__e=event"/>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
function MessageDetail($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { message, onBack } = $$props;
    function sanitizeHTML(html2) {
      return DOMPurify.sanitize(html2, {
        ALLOWED_TAGS: [
          "p",
          "br",
          "strong",
          "em",
          "u",
          "h1",
          "h2",
          "h3",
          "h4",
          "h5",
          "h6",
          "ul",
          "ol",
          "li",
          "a",
          "blockquote",
          "code",
          "pre",
          "img",
          "div",
          "span"
        ],
        ALLOWED_ATTR: ["href", "src", "alt", "title", "class", "style"]
      });
    }
    if (message) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="h-full flex flex-col bg-white dark:bg-gray-800"><div class="flex-shrink-0 px-4 pt-3 pb-3 border-b border-gray-200 dark:border-gray-700 flex items-center gap-2">`);
      Button($$renderer2, {
        variant: "ghost",
        size: "icon",
        onclick: onBack,
        children: ($$renderer3) => {
          Arrow_left($$renderer3, { class: "h-5 w-5" });
        }
      });
      $$renderer2.push(`<!----> <div class="flex items-center gap-1">`);
      Button($$renderer2, {
        variant: "ghost",
        size: "sm",
        children: ($$renderer3) => {
          Archive($$renderer3, { class: "h-4 w-4" });
        }
      });
      $$renderer2.push(`<!----> `);
      Button($$renderer2, {
        variant: "ghost",
        size: "sm",
        children: ($$renderer3) => {
          Trash_2($$renderer3, { class: "h-4 w-4" });
        }
      });
      $$renderer2.push(`<!----> `);
      Button($$renderer2, {
        variant: "ghost",
        size: "sm",
        children: ($$renderer3) => {
          Star($$renderer3, {
            class: message.is_starred ? "h-4 w-4 fill-yellow-400 text-yellow-400" : "h-4 w-4"
          });
        }
      });
      $$renderer2.push(`<!----> `);
      Button($$renderer2, {
        variant: "ghost",
        size: "sm",
        children: ($$renderer3) => {
          Ellipsis_vertical($$renderer3, { class: "h-4 w-4" });
        }
      });
      $$renderer2.push(`<!----></div> <div class="flex-1"></div> <div class="flex items-center gap-1">`);
      Button($$renderer2, {
        variant: "ghost",
        size: "sm",
        children: ($$renderer3) => {
          Reply($$renderer3, { class: "h-4 w-4" });
          $$renderer3.push(`<!----> <span>Reply</span>`);
        }
      });
      $$renderer2.push(`<!----> `);
      Button($$renderer2, {
        variant: "ghost",
        size: "sm",
        children: ($$renderer3) => {
          Reply_all($$renderer3, { class: "h-4 w-4" });
          $$renderer3.push(`<!----> <span>Reply all</span>`);
        }
      });
      $$renderer2.push(`<!----> `);
      Button($$renderer2, {
        variant: "ghost",
        size: "sm",
        children: ($$renderer3) => {
          Forward($$renderer3, { class: "h-4 w-4" });
          $$renderer3.push(`<!----> <span>Forward</span>`);
        }
      });
      $$renderer2.push(`<!----></div></div> <div class="flex-1 overflow-y-auto custom-scrollbar"><div class="px-8 py-6"><h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-6">${escape_html(message.subject || "(no subject)")}</h1> <div class="flex items-start gap-4 mb-8">`);
      Avatar($$renderer2, {
        email: message.from.email,
        name: message.from.name,
        size: "lg"
      });
      $$renderer2.push(`<!----> <div class="flex-1"><div class="flex items-center justify-between"><div><div class="font-semibold text-gray-900 dark:text-gray-100">${escape_html(message.from.name || message.from.email)}</div> <div class="text-sm text-gray-600 dark:text-gray-400">${escape_html(message.from.email)}</div></div> <div class="text-sm text-gray-600 dark:text-gray-400">${escape_html(formatTimestamp(message.received_at))}</div></div> <div class="mt-3 text-sm text-gray-600 dark:text-gray-400 space-y-1">`);
      if (message.to && message.to.length > 0) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="flex gap-2"><span class="font-medium min-w-8">To:</span> <span>${escape_html(message.to.map((t) => t.email).join(", "))}</span></div>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> `);
      if (message.cc && message.cc.length > 0) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="flex gap-2"><span class="font-medium min-w-8">Cc:</span> <span>${escape_html(message.cc.map((c) => c.email).join(", "))}</span></div>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div></div></div> <div class="prose dark:prose-invert max-w-none prose-sm prose-blue prose-img:rounded-lg prose-a:text-primary-600 dark:prose-a:text-primary-400">`);
      if (message.body_html) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`${html(sanitizeHTML(message.body_html))}`);
      } else {
        $$renderer2.push("<!--[!-->");
        if (message.body_text) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<pre class="whitespace-pre-wrap font-sans">${escape_html(message.body_text)}</pre>`);
        } else {
          $$renderer2.push("<!--[!-->");
          $$renderer2.push(`<p class="text-gray-500 dark:text-gray-400 italic">No message body</p>`);
        }
        $$renderer2.push(`<!--]-->`);
      }
      $$renderer2.push(`<!--]--></div> `);
      if (message.has_attachments) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="mt-8 p-4 border border-gray-200 dark:border-gray-700 rounded-lg bg-gray-50 dark:bg-gray-700/50"><div class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300 mb-2">`);
        Download($$renderer2, { class: "h-4 w-4" });
        $$renderer2.push(`<!----> <span class="font-medium">Attachments</span></div> <div class="text-sm text-gray-500 dark:text-gray-400">Attachment support coming in Phase 6</div></div>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div></div></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]-->`);
  });
}
function Input($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let {
      type = "text",
      placeholder = "",
      value = "",
      class: className,
      disabled = false,
      required = false,
      oninput
    } = $$props;
    $$renderer2.push(`<input${attr("type", type)}${attr("placeholder", placeholder)}${attr("disabled", disabled, true)}${attr("required", required, true)}${attr("value", value)}${attr_class(clsx(cn("flex h-9 w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1 text-sm text-gray-900 dark:text-gray-100 placeholder:text-gray-500 dark:placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-50", className)))}/>`);
    bind_props($$props, { value });
  });
}
function RichTextEditor($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let {
      content = "",
      placeholder = "Write your message...",
      class: className,
      onUpdate
    } = $$props;
    let editor = null;
    onDestroy(() => {
    });
    function getContent() {
      return { html: "", text: "" };
    }
    $$renderer2.push(`<div${attr_class(clsx(
      // Expose methods for parent component
      cn("overflow-hidden", className)
    ))}><div class="px-4 pt-2"><div class="flex items-center gap-1 px-3 py-1.5 rounded-md w-fit" style="background-color: #F1F4FA;"><button${attr_class(clsx(cn("p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors", editor?.isActive("bold"))))}>`);
    Bold($$renderer2, { class: "h-3.5 w-3.5 text-gray-700 dark:text-gray-300" });
    $$renderer2.push(`<!----></button> <button${attr_class(clsx(cn("p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors", editor?.isActive("italic"))))}>`);
    Italic($$renderer2, { class: "h-3.5 w-3.5 text-gray-700 dark:text-gray-300" });
    $$renderer2.push(`<!----></button> <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div> <button${attr_class(clsx(cn("p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors", editor?.isActive("bulletList"))))}>`);
    List($$renderer2, { class: "h-3.5 w-3.5 text-gray-700 dark:text-gray-300" });
    $$renderer2.push(`<!----></button> <button${attr_class(clsx(cn("p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors", editor?.isActive("orderedList"))))}>`);
    List_ordered($$renderer2, { class: "h-3.5 w-3.5 text-gray-700 dark:text-gray-300" });
    $$renderer2.push(`<!----></button> <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div> <button${attr_class(clsx(cn("p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors", editor?.isActive("link"))))}>`);
    Link_2($$renderer2, { class: "h-3.5 w-3.5 text-gray-700 dark:text-gray-300" });
    $$renderer2.push(`<!----></button> <button${attr_class(clsx(cn("p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors", editor?.isActive("codeBlock"))))}>`);
    Code($$renderer2, { class: "h-3.5 w-3.5 text-gray-700 dark:text-gray-300" });
    $$renderer2.push(`<!----></button></div></div> <div class="bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100"></div></div>`);
    bind_props($$props, { getContent });
  });
}
function ContactAutocomplete($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let {
      value = "",
      placeholder = "Recipients (comma-separated)",
      class: className,
      disabled = false,
      required = false
    } = $$props;
    $$renderer2.push(`<div${attr_class(clsx(cn("relative", className)))}><input type="text"${attr("placeholder", placeholder)}${attr("disabled", disabled, true)}${attr("required", required, true)}${attr("value", value)} class="flex h-9 w-full px-0 py-1 text-sm text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"/> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div>`);
    bind_props($$props, { value });
  });
}
const API_URL = "http://localhost:3001";
class EmailAPIClient {
  client;
  constructor() {
    this.client = axios.create({
      baseURL: API_URL,
      headers: {
        "Content-Type": "application/json"
      }
    });
  }
  // Account
  async getAccount(kratosId) {
    const { data } = await this.client.get(`/api/mail/account`, {
      params: { kratos_id: kratosId }
    });
    return data;
  }
  // Mailboxes
  async getMailboxes(accountId) {
    const { data } = await this.client.get(`/api/mail/mailboxes`, {
      params: { account_id: accountId }
    });
    return data.mailboxes;
  }
  async createMailbox(accountId, name, parentId) {
    const { data } = await this.client.post(`/api/mail/mailboxes`, {
      account_id: accountId,
      name,
      parent_id: parentId
    });
    return data;
  }
  // Messages
  async getMessages(accountId, mailboxId, limit = 50, offset = 0) {
    const { data } = await this.client.get(`/api/mail/messages`, {
      params: { account_id: accountId, mailbox_id: mailboxId, limit, offset }
    });
    return data;
  }
  async getMessage(messageId, accountId) {
    const { data } = await this.client.get(`/api/mail/messages/${messageId}`, {
      params: { account_id: accountId }
    });
    return data.message;
  }
  async sendEmail(request, accountId) {
    const { data } = await this.client.post(`/api/mail/messages`, {
      ...request,
      account_id: accountId
    });
    return data;
  }
  // Search
  async searchEmails(params) {
    const { data } = await this.client.get(`/api/mail/search`, { params });
    return data;
  }
  // WebSocket token
  async getWebSocketToken(userId) {
    const { data } = await this.client.get(`/api/mail/ws/token`, {
      params: { user_id: userId }
    });
    return data;
  }
  // Health check
  async health() {
    const { data } = await this.client.get(`/health`);
    return data;
  }
}
const emailAPI = new EmailAPIClient();
class EmailStore {
  currentMailbox = "inbox";
  messages = [];
  selectedMessage = null;
  mailboxes = [];
  loading = false;
  error = null;
  unreadCount = 0;
  // Mock account ID for Phase 4 (will come from auth in production)
  accountId = "test-account-123";
  // User ID for realtime connection (will come from auth in production)
  userId = "test-user-123";
  async loadMailboxes() {
    this.loading = true;
    this.error = null;
    try {
      this.mailboxes = await emailAPI.getMailboxes(this.accountId);
    } catch (err) {
      this.error = err instanceof Error ? err.message : "Failed to load mailboxes";
      console.error("Error loading mailboxes:", err);
    } finally {
      this.loading = false;
    }
  }
  async loadMessages(mailboxId, limit = 50) {
    this.loading = true;
    this.error = null;
    this.currentMailbox = mailboxId;
    try {
      const result = await emailAPI.getMessages(this.accountId, mailboxId, limit);
      this.messages = result.messages;
    } catch (err) {
      this.error = err instanceof Error ? err.message : "Failed to load messages";
      console.error("Error loading messages:", err);
    } finally {
      this.loading = false;
    }
  }
  async loadMessage(messageId) {
    this.loading = true;
    this.error = null;
    try {
      const message = await emailAPI.getMessage(messageId, this.accountId);
      this.selectedMessage = message;
    } catch (err) {
      this.error = err instanceof Error ? err.message : "Failed to load message";
      console.error("Error loading message:", err);
    } finally {
      this.loading = false;
    }
  }
  selectMessage(message) {
    this.selectedMessage = message;
  }
  clearSelection() {
    this.selectedMessage = null;
  }
  // Send email
  async sendEmail(to, subject, bodyText) {
    this.loading = true;
    this.error = null;
    try {
      const result = await emailAPI.sendEmail({ to, subject, body_text: bodyText }, this.accountId);
      return result.message_id;
    } catch (err) {
      this.error = err instanceof Error ? err.message : "Failed to send email";
      console.error("Error sending email:", err);
      throw err;
    } finally {
      this.loading = false;
    }
  }
  // Search emails
  async searchEmails(query) {
    this.loading = true;
    this.error = null;
    try {
      const result = await emailAPI.searchEmails({ query, account_id: this.accountId, limit: 50 });
      this.messages = result.results;
    } catch (err) {
      this.error = err instanceof Error ? err.message : "Failed to search emails";
      console.error("Error searching emails:", err);
    } finally {
      this.loading = false;
    }
  }
  // =====================
  // Real-time event handlers
  // =====================
  /**
   * Handle new email event from Centrifugo
   * Adds the new email to the top of the message list if in inbox
   */
  async handleNewEmail(emailId, from, subject, preview) {
    if (this.currentMailbox === "inbox") {
      const newEmail = {
        id: emailId,
        subject,
        from: { email: from },
        preview,
        received_at: /* @__PURE__ */ (/* @__PURE__ */ new Date()).toISOString(),
        is_read: false,
        is_starred: false,
        has_attachments: false
      };
      this.messages = [newEmail, ...this.messages];
    }
    this.unreadCount++;
    this.loadMailboxes();
  }
  /**
   * Handle email updated event from Centrifugo
   */
  handleEmailUpdated(emailId, updateType) {
    const messageIndex = this.messages.findIndex((m) => m.id === emailId);
    if (messageIndex === -1) return;
    const message = this.messages[messageIndex];
    switch (updateType) {
      case "read":
        this.messages[messageIndex] = { ...message, is_read: true };
        if (!message.is_read) this.unreadCount--;
        break;
      case "unread":
        this.messages[messageIndex] = { ...message, is_read: false };
        if (message.is_read) this.unreadCount++;
        break;
      case "starred":
        this.messages[messageIndex] = { ...message, is_starred: true };
        break;
      case "unstarred":
        this.messages[messageIndex] = { ...message, is_starred: false };
        break;
      case "deleted":
      case "moved":
        this.messages = this.messages.filter((m) => m.id !== emailId);
        if (!message.is_read) this.unreadCount--;
        break;
    }
    if (this.selectedMessage?.id === emailId) {
      if (updateType === "deleted" || updateType === "moved") {
        this.selectedMessage = null;
      } else {
        this.selectedMessage = this.messages[messageIndex];
      }
    }
  }
  /**
   * Handle mailbox updated event from Centrifugo
   */
  handleMailboxUpdated(mailboxId, action) {
    this.loadMailboxes();
    if (this.currentMailbox === mailboxId) {
      this.loadMessages(mailboxId);
    }
  }
  /**
   * Update unread count from mailboxes
   */
  updateUnreadCount() {
    const inbox = this.mailboxes.find((m) => m.role === "inbox" || m.name.toLowerCase() === "inbox");
    this.unreadCount = inbox?.unread_emails || 0;
  }
}
const emailStore = new EmailStore();
function Composer($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let {
      open = false,
      onClose,
      replyTo = "",
      replySubject = "",
      replyBody = ""
    } = $$props;
    let to = replyTo;
    let subject = replySubject;
    let sending = false;
    let initialContent = "";
    if (replyBody) {
      initialContent = `<br><br><blockquote class="border-l-4 border-gray-300 pl-4 text-gray-600">${replyBody}</blockquote>`;
    }
    async function handleSend() {
      if (!to.trim()) {
        alert("Please enter at least one recipient");
        return;
      }
      if (!subject.trim()) {
        if (!confirm("Send this message without a subject?")) {
          return;
        }
      }
      {
        alert("Please enter a message");
        return;
      }
    }
    let $$settled = true;
    let $$inner_renderer;
    function $$render_inner($$renderer3) {
      if (open) {
        $$renderer3.push("<!--[-->");
        $$renderer3.push(`<div${attr_class(clsx(cn("fixed inset-0 z-50", "pointer-events-none")))} role="dialog" aria-modal="true"><div${attr_class(clsx(cn("bg-white dark:bg-gray-800 overflow-hidden pointer-events-auto flex flex-col shadow-lg", "absolute bottom-0 right-[30px] w-[540px] rounded-t-lg h-[calc(100vh-100px)] max-h-[700px]")))}><div class="flex-shrink-0 flex items-center justify-between px-4 py-2" style="background-color: #F1F4FA;"><h2 class="text-sm font-medium text-gray-900 dark:text-gray-100">New message</h2> <div class="flex items-center gap-1">`);
        {
          $$renderer3.push("<!--[!-->");
        }
        $$renderer3.push(`<!--]--> <button class="p-1 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-colors"${attr("title", "Expand")}>`);
        {
          $$renderer3.push("<!--[!-->");
          Maximize_2($$renderer3, { class: "h-4 w-4 text-gray-600 dark:text-gray-300" });
        }
        $$renderer3.push(`<!--]--></button> <button class="p-1 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-colors" title="Close">`);
        X($$renderer3, { class: "h-4 w-4 text-gray-600 dark:text-gray-300" });
        $$renderer3.push(`<!----></button></div></div> <div class="flex-1 overflow-y-auto"><div class="space-y-0"><div class="py-1 pl-4"><div class="flex items-center pb-1 border-b border-gray-200 dark:border-gray-700">`);
        ContactAutocomplete($$renderer3, {
          placeholder: "To",
          class: "flex-1 bg-transparent",
          get value() {
            return to;
          },
          set value($$value) {
            to = $$value;
            $$settled = false;
          }
        });
        $$renderer3.push(`<!----> <button class="text-xs text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 px-4 flex-shrink-0">Cc</button></div></div> `);
        {
          $$renderer3.push("<!--[!-->");
        }
        $$renderer3.push(`<!--]--> <div class="py-1 px-4"><div class="pb-1 border-b border-gray-200 dark:border-gray-700">`);
        Input($$renderer3, {
          type: "text",
          placeholder: "Subject",
          class: "w-full border-0 focus:ring-0 px-0 bg-transparent h-auto py-0",
          get value() {
            return subject;
          },
          set value($$value) {
            subject = $$value;
            $$settled = false;
          }
        });
        $$renderer3.push(`<!----></div></div> <div>`);
        RichTextEditor($$renderer3, { content: initialContent });
        $$renderer3.push(`<!----></div></div></div> <div class="flex-shrink-0 px-4 py-3 flex items-center justify-between" style="background-color: #F1F4FA;"><div class="flex items-center gap-2">`);
        Button($$renderer3, {
          variant: "primary",
          onclick: handleSend,
          disabled: sending,
          class: "text-sm",
          children: ($$renderer4) => {
            $$renderer4.push(`<!---->${escape_html("Send")}`);
          }
        });
        $$renderer3.push(`<!----> <button disabled class="p-2 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed" title="Attach files (coming soon)">`);
        Paperclip($$renderer3, { class: "h-4 w-4 text-gray-600 dark:text-gray-300" });
        $$renderer3.push(`<!----></button></div> <div class="text-xs text-gray-500 dark:text-gray-400">`);
        {
          $$renderer3.push("<!--[!-->");
          {
            $$renderer3.push("<!--[!-->");
          }
          $$renderer3.push(`<!--]-->`);
        }
        $$renderer3.push(`<!--]--></div></div></div></div>`);
      } else {
        $$renderer3.push("<!--[!-->");
      }
      $$renderer3.push(`<!--]-->`);
    }
    do {
      $$settled = true;
      $$inner_renderer = $$renderer2.copy();
      $$render_inner($$inner_renderer);
    } while (!$$settled);
    $$renderer2.subsume($$inner_renderer);
    bind_props($$props, { open });
  });
}
const CENTRIFUGO_URL = "ws://localhost:8001/connection/websocket";
class RealtimeStore {
  centrifuge = null;
  subscription = null;
  handlers = {};
  userId = null;
  reconnectAttempts = 0;
  maxReconnectAttempts = 5;
  // State using Svelte 5 runes
  connected = false;
  connecting = false;
  error = null;
  lastEvent = null;
  notificationsEnabled = false;
  /**
   * Initialize and connect to Centrifugo
   */
  async connect(userId, handlers = {}) {
    if (this.centrifuge && this.connected) {
      console.log("Already connected to Centrifugo");
      return;
    }
    this.userId = userId;
    this.handlers = handlers;
    this.connecting = true;
    this.error = null;
    try {
      const { token, channel } = await emailAPI.getWebSocketToken(userId);
      this.centrifuge = new Centrifuge(CENTRIFUGO_URL, { token, debug: false });
      this.setupConnectionHandlers();
      this.centrifuge.connect();
      await this.subscribeToChannel(channel);
      await this.requestNotificationPermission();
      console.log(`Connected to Centrifugo, subscribed to ${channel}`);
    } catch (err) {
      this.error = err instanceof Error ? err.message : "Failed to connect to Centrifugo";
      console.error("Centrifugo connection error:", err);
      this.connecting = false;
    }
  }
  /**
   * Setup Centrifuge connection event handlers
   */
  setupConnectionHandlers() {
    if (!this.centrifuge) return;
    this.centrifuge.on("connected", () => {
      this.connected = true;
      this.connecting = false;
      this.reconnectAttempts = 0;
      this.error = null;
      this.handlers.onConnectionStateChange?.(true);
      console.log("Centrifugo connected");
    });
    this.centrifuge.on("disconnected", () => {
      this.connected = false;
      this.handlers.onConnectionStateChange?.(false);
      console.log("Centrifugo disconnected");
    });
    this.centrifuge.on("error", (ctx) => {
      this.error = ctx.error?.message || "Connection error";
      console.error("Centrifugo error:", ctx);
    });
  }
  /**
   * Subscribe to user's email channel
   */
  async subscribeToChannel(channel) {
    if (!this.centrifuge) return;
    this.subscription = this.centrifuge.newSubscription(channel);
    this.subscription.on("publication", (ctx) => {
      this.handlePublication(ctx.data);
    });
    this.subscription.on("subscribed", () => {
      console.log(`Subscribed to channel: ${channel}`);
    });
    this.subscription.on("error", (ctx) => {
      console.error(`Subscription error for ${channel}:`, ctx);
    });
    this.subscription.subscribe();
  }
  /**
   * Handle incoming publication events
   */
  handlePublication(event) {
    this.lastEvent = event;
    switch (event.type) {
      case "new_email":
        this.handleNewEmail(event);
        break;
      case "email_updated":
        this.handleEmailUpdated(event);
        break;
      case "mailbox_updated":
        this.handleMailboxUpdated(event);
        break;
      default:
        console.warn("Unknown event type:", event);
    }
  }
  /**
   * Handle new email event
   */
  handleNewEmail(event) {
    console.log("New email received:", event);
    this.handlers.onNewEmail?.(event);
    this.showNotification(`New email from ${event.from}`, event.subject, event.preview);
  }
  /**
   * Handle email updated event
   */
  handleEmailUpdated(event) {
    console.log("Email updated:", event);
    this.handlers.onEmailUpdated?.(event);
  }
  /**
   * Handle mailbox updated event
   */
  handleMailboxUpdated(event) {
    console.log("Mailbox updated:", event);
    this.handlers.onMailboxUpdated?.(event);
  }
  /**
   * Request notification permission
   */
  async requestNotificationPermission() {
    if (typeof window === "undefined" || !("Notification" in window)) {
      console.log("Notifications not supported");
      return false;
    }
    if (Notification.permission === "granted") {
      this.notificationsEnabled = true;
      return true;
    }
    if (Notification.permission !== "denied") {
      const permission = await Notification.requestPermission();
      this.notificationsEnabled = permission === "granted";
      return this.notificationsEnabled;
    }
    return false;
  }
  /**
   * Show desktop notification
   */
  showNotification(title, subject, body) {
    if (!this.notificationsEnabled || typeof window === "undefined") return;
    try {
      const notification = new Notification(title, {
        body: `${subject}
${body.substring(0, 100)}...`,
        icon: "/arackmail.svg",
        tag: "arack-mail",
        requireInteraction: false
      });
      setTimeout(() => notification.close(), 5e3);
      notification.onclick = () => {
        window.focus();
        notification.close();
      };
    } catch (err) {
      console.error("Failed to show notification:", err);
    }
  }
  /**
   * Disconnect from Centrifugo
   */
  disconnect() {
    if (this.subscription) {
      this.subscription.unsubscribe();
      this.subscription = null;
    }
    if (this.centrifuge) {
      this.centrifuge.disconnect();
      this.centrifuge = null;
    }
    this.connected = false;
    this.connecting = false;
    this.userId = null;
    this.handlers = {};
    console.log("Disconnected from Centrifugo");
  }
  /**
   * Check if connected
   */
  isConnected() {
    return this.connected;
  }
  /**
   * Register event handlers
   */
  setHandlers(handlers) {
    this.handlers = { ...this.handlers, ...handlers };
  }
  /**
   * Reconnect with current user
   */
  async reconnect() {
    if (this.userId) {
      this.disconnect();
      await this.connect(this.userId, this.handlers);
    }
  }
}
const realtimeStore = new RealtimeStore();
export {
  Button as B,
  Composer as C,
  Input as I,
  Moon as M,
  Search as S,
  Wifi as W,
  Wifi_off as a,
  Settings as b,
  MailboxList as c,
  MessageDetail as d,
  emailStore as e,
  MessageList as f,
  Sun as g,
  onDestroy as o,
  realtimeStore as r
};
