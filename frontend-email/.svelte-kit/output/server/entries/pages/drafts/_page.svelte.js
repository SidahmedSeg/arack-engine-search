import { w as head, x as attr_class, y as stringify } from "../../../chunks/index2.js";
import { o as onDestroy, S as Search, I as Input, r as realtimeStore, W as Wifi, a as Wifi_off, B as Button, M as Moon, b as Settings, c as MailboxList, e as emailStore, d as MessageDetail, f as MessageList, C as Composer, g as Sun } from "../../../chunks/realtime.svelte.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let searchQuery = "";
    let darkMode = false;
    let composerOpen = false;
    onDestroy(() => {
      realtimeStore.disconnect();
    });
    function toggleDarkMode() {
      darkMode = !darkMode;
      if (typeof window !== "undefined" && window.toggleDarkMode) {
        window.toggleDarkMode();
      }
    }
    async function handleMailboxSelect(mailboxId) {
      await emailStore.loadMessages(mailboxId);
      emailStore.clearSelection();
    }
    function handleMessageSelect(message) {
      emailStore.selectMessage(message);
    }
    function handleBackToList() {
      emailStore.clearSelection();
    }
    function handleComposerClose() {
      composerOpen = false;
      emailStore.loadMessages(emailStore.currentMailbox);
    }
    let $$settled = true;
    let $$inner_renderer;
    function $$render_inner($$renderer3) {
      head("1nln88x", $$renderer3, ($$renderer4) => {
        $$renderer4.title(($$renderer5) => {
          $$renderer5.push(`<title>Drafts - Arack Mail</title>`);
        });
      });
      $$renderer3.push(`<div class="h-screen flex flex-col overflow-hidden bg-gray-50 dark:bg-gray-900"><header class="flex-shrink-0 h-16" style="background-color: #F8FAFD;"><div class="h-full px-6 flex items-center gap-4"><div class="flex items-center"><img src="/arackmail.svg" alt="Arack Mail" class="h-6"/></div> <form class="flex-1 max-w-2xl mx-4"><div class="relative">`);
      Search($$renderer3, {
        class: "absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400"
      });
      $$renderer3.push(`<!----> `);
      Input($$renderer3, {
        type: "search",
        placeholder: "Search mail",
        class: "pl-10 w-full bg-white dark:bg-gray-700 border-transparent focus:ring-2 focus:ring-primary-500 rounded-full",
        get value() {
          return searchQuery;
        },
        set value($$value) {
          searchQuery = $$value;
          $$settled = false;
        }
      });
      $$renderer3.push(`<!----></div></form> <div class="flex items-center gap-2 ml-auto"><div${attr_class(`flex items-center gap-1 px-2 py-1 rounded-full text-xs ${stringify(realtimeStore.connected ? "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300" : realtimeStore.connecting ? "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300" : "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300")}`)}>`);
      if (realtimeStore.connected) {
        $$renderer3.push("<!--[-->");
        Wifi($$renderer3, { class: "h-3 w-3" });
        $$renderer3.push(`<!----> <span>Live</span>`);
      } else {
        $$renderer3.push("<!--[!-->");
        if (realtimeStore.connecting) {
          $$renderer3.push("<!--[-->");
          Wifi($$renderer3, { class: "h-3 w-3 animate-pulse" });
          $$renderer3.push(`<!----> <span>Connecting...</span>`);
        } else {
          $$renderer3.push("<!--[!-->");
          Wifi_off($$renderer3, { class: "h-3 w-3" });
          $$renderer3.push(`<!----> <span>Offline</span>`);
        }
        $$renderer3.push(`<!--]-->`);
      }
      $$renderer3.push(`<!--]--></div> `);
      Button($$renderer3, {
        variant: "ghost",
        size: "icon",
        onclick: toggleDarkMode,
        children: ($$renderer4) => {
          if (darkMode) {
            $$renderer4.push("<!--[-->");
            Sun($$renderer4, { class: "h-5 w-5" });
          } else {
            $$renderer4.push("<!--[!-->");
            Moon($$renderer4, { class: "h-5 w-5" });
          }
          $$renderer4.push(`<!--]-->`);
        }
      });
      $$renderer3.push(`<!----> `);
      Button($$renderer3, {
        variant: "ghost",
        size: "icon",
        children: ($$renderer4) => {
          Settings($$renderer4, { class: "h-5 w-5" });
        }
      });
      $$renderer3.push(`<!----></div></div></header> <div class="flex-1 flex overflow-hidden">`);
      MailboxList($$renderer3, {
        mailboxes: emailStore.mailboxes,
        currentMailbox: emailStore.currentMailbox,
        onMailboxSelect: handleMailboxSelect,
        onCompose: () => composerOpen = true
      });
      $$renderer3.push(`<!----> <div class="flex-1 overflow-hidden pt-4 pr-6 pb-6"><div class="h-full bg-white dark:bg-gray-800 rounded-2xl shadow-sm overflow-hidden">`);
      if (emailStore.selectedMessage) {
        $$renderer3.push("<!--[-->");
        MessageDetail($$renderer3, {
          message: emailStore.selectedMessage,
          onBack: handleBackToList
        });
      } else {
        $$renderer3.push("<!--[!-->");
        MessageList($$renderer3, {
          messages: emailStore.messages,
          selectedMessage: emailStore.selectedMessage,
          onMessageSelect: handleMessageSelect,
          loading: emailStore.loading
        });
      }
      $$renderer3.push(`<!--]--></div></div></div> `);
      Composer($$renderer3, {
        onClose: handleComposerClose,
        get open() {
          return composerOpen;
        },
        set open($$value) {
          composerOpen = $$value;
          $$settled = false;
        }
      });
      $$renderer3.push(`<!----> <div class="fixed bottom-4 right-4 text-xs text-gray-500 dark:text-gray-400"><details class="bg-white dark:bg-gray-800 p-2 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700"><summary class="cursor-pointer font-medium">Keyboard shortcuts</summary> <div class="mt-2 space-y-1"><div class="flex justify-between gap-4"><span>Compose</span> <kbd class="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">c</kbd></div> <div class="flex justify-between gap-4"><span>Next message</span> <kbd class="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">j</kbd></div> <div class="flex justify-between gap-4"><span>Previous message</span> <kbd class="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">k</kbd></div> <div class="flex justify-between gap-4"><span>Search</span> <kbd class="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">/</kbd></div></div></details></div></div>`);
    }
    do {
      $$settled = true;
      $$inner_renderer = $$renderer2.copy();
      $$render_inner($$inner_renderer);
    } while (!$$settled);
    $$renderer2.subsume($$inner_renderer);
  });
}
export {
  _page as default
};
