import JSX from "./createElement";

export interface ListObserver {
  on_click(elem: number): void
}

export class List {
  private element: HTMLElement;
  private elements: Map<number, HTMLElement> = new Map();
  private observer: ListObserver;
  private id = 0;

  public constructor(observer: ListObserver, path: String) {
    this.observer = observer;
    this.element = <div></div>;
    document.body.appendChild(this.element);
  }

  public add_element(content: string): number {
    const new_element: HTMLDivElement = <div>{content}</div>
    const current_id = this.id;
    new_element.addEventListener("click", _ => { this.observer.on_click(current_id) })
    this.elements[current_id] = new_element;
    this.element.appendChild(new_element);
    this.id++;
    return current_id;
  }

  public remove_element(id: number) {
    this.elements.get(id)?.remove();
  }
}