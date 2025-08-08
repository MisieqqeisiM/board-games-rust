export interface SocketObserver {
    on_data(data: Uint8Array): void;
}

export class Socket {
    private socket: WebSocket;

    public constructor(observer: SocketObserver, path: String) {
        this.socket = new WebSocket(`ws://${location.host}/${path}`);
        this.socket.addEventListener("message", async e => {
            const data: Blob = e.data;
            observer.on_data(new Uint8Array(await data.arrayBuffer()))
        });
    }

    public send(data: Uint8Array) {
        this.socket.send(data.buffer);
    }
}