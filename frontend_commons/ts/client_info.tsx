import JSX from "./createElement";

export interface ClientData {
  name: string,
  ping: number,
}

export class ClientInfo {
  private element: HTMLElement;
  private clientData: ClientData

  public constructor(clientData: ClientData) {
    this.clientData = clientData;
    this.element = <div></div>;
    this.updateElement();
    document.body.appendChild(this.element);
  }

  private updateElement() {
    this.element.innerText = `${this.clientData.name} (${this.clientData.ping}ms)`;
  }

  public setPing(ping: number) {
    this.clientData.ping = ping;
    this.updateElement();
  }
}