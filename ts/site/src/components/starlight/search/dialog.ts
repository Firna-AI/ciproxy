import { SearchModel } from "./model.ts";
import { searchPagefind } from "./pagefind.ts";
import { element, SearchView } from "./view.ts";

export class DocsSearch extends HTMLElement {
  private events?: AbortController;
  private close?: () => void;

  connectedCallback(): void {
    this.events?.abort();
    this.events = new AbortController();
    const { signal } = this.events;
    const dialog = element<HTMLDialogElement>(this, "dialog");
    const openButton = element<HTMLButtonElement>(this, "[data-open-modal]");
    const closeButton = element<HTMLButtonElement>(this, "[data-close-modal]");
    const clearButton = element<HTMLButtonElement>(this, ".ds-clear");
    const input = element<HTMLInputElement>(this, "input");
    let returnFocus: HTMLElement = openButton;
    const view = new SearchView(
      this,
      (index) => model.select(index),
      (event) => {
        if (
          !event.metaKey &&
          !event.ctrlKey &&
          !event.shiftKey &&
          !event.altKey
        ) {
          dialog.close();
        }
      },
    );
    const model = new SearchModel(searchPagefind, (state) =>
      view.render(state),
    );
    view.render(model.state);

    const open = () => {
      if (dialog.open) return;
      returnFocus =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : openButton;
      dialog.showModal();
      openButton.setAttribute("aria-expanded", "true");
      document.body.setAttribute("data-search-modal-open", "");
      input.focus();
      if (model.state.status === "error") void model.search(input.value);
    };
    this.close = () => dialog.close();
    openButton.disabled = false;
    openButton.addEventListener("click", open, { signal });
    closeButton.addEventListener("click", this.close, { signal });
    dialog.addEventListener(
      "close",
      () => {
        document.body.removeAttribute("data-search-modal-open");
        openButton.setAttribute("aria-expanded", "false");
        returnFocus.focus();
      },
      { signal },
    );

    const update = () => {
      clearButton.hidden = input.value.length === 0;
      this.classList.remove("has-keyboard-selection");
      void model.search(input.value);
    };
    input.addEventListener("input", update, { signal });
    clearButton.addEventListener(
      "click",
      () => {
        input.value = "";
        update();
        input.focus();
      },
      { signal },
    );
    input.addEventListener("compositionend", update, { signal });
    dialog.addEventListener(
      "keydown",
      (event) => {
        if (event.isComposing || event.metaKey || event.ctrlKey || event.altKey)
          return;
        const target = event.target;
        if (target !== input && !(target instanceof HTMLAnchorElement)) return;
        if (event.key === "ArrowDown" || event.key === "ArrowUp") {
          event.preventDefault();
          this.classList.add("has-keyboard-selection");
          model.move(event.key === "ArrowDown" ? 1 : -1);
          input.focus();
          if (model.state.status === "ready") {
            view.scrollActive(model.state.active);
          }
        } else if (event.key === "Enter" && target === input) {
          event.preventDefault();
          if (model.state.status === "ready")
            view.links[model.state.active]?.click();
        }
      },
      { signal },
    );
    dialog.addEventListener(
      "click",
      (event) => {
        if (event.target !== dialog) return;
        const bounds = dialog.getBoundingClientRect();
        if (
          event.clientX < bounds.left ||
          event.clientX > bounds.right ||
          event.clientY < bounds.top ||
          event.clientY > bounds.bottom
        )
          dialog.close();
      },
      { signal },
    );
    window.addEventListener(
      "keydown",
      (event) => {
        if (
          (event.metaKey || event.ctrlKey) &&
          event.key.toLowerCase() === "k"
        ) {
          event.preventDefault();
          if (dialog.open) dialog.close();
          else open();
        }
      },
      { signal },
    );
    const shortcut = element(this, ".search-shortcut");
    shortcut.textContent = /Mac|iPhone|iPad|iPod/.test(navigator.platform)
      ? "⌘ K"
      : "Ctrl K";
  }

  disconnectedCallback(): void {
    this.close?.();
    document.body.removeAttribute("data-search-modal-open");
    this.events?.abort();
  }
}
