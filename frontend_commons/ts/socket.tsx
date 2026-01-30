export interface SocketObserver {
    on_data(data: Uint8Array): void;
    on_close(): void;
    on_error(): void;
}

export class Socket {
    private socket: WebSocket;

    public constructor(observer: SocketObserver, path: String) {
        const protocol = location.protocol === "http:" ? "ws:" : "wss:";
        this.socket = new WebSocket(`${protocol}//${location.host}${location.pathname}/${path}`);
        this.socket.addEventListener("message", async e => {
            const data: Blob = e.data;
            observer.on_data(new Uint8Array(await data.arrayBuffer()))
        });
        this.socket.addEventListener("error", async _ => {
            observer.on_error();
        });
        this.socket.addEventListener("close", async _ => {
            observer.on_close();
        });
    }

    public send(data: Uint8Array) {
        this.socket.send(data.buffer);
    }
}