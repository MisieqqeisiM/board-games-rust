function nonNull<T>(val: T, fallback: T) { return Boolean(val) ? val : fallback };

function DOMparseChildren(children: (HTMLElement | string)[]): (HTMLElement | Text)[] {
  return children.map(child => {
    if (typeof child === 'string') {
      return document.createTextNode(child);
    }
    return child;
  })
}

function DOMparseNode(element: string, properties: object, children: HTMLElement[]) {
  const el = document.createElement(element);
  Object.keys(nonNull(properties, {})).forEach(key => {
    el[key] = properties[key];
  })
  DOMparseChildren(children).forEach(child => {
    el.appendChild(child);
  });
  return el;
}

const JSX = {
  createElement(element: string, properties: object, ...children: HTMLElement[]): HTMLElement {
    return DOMparseNode(element, properties, children);
  }
}

export default JSX;