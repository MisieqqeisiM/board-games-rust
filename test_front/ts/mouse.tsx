export interface MouseObserver {
    on_move(x: number, y: number): void;
    on_down(button: number, x: number, y: number): void;
    on_up(button: number, x: number, y: number): void;
    on_scroll(delta_x: number, delta_y: number): void;
}

export class Mouse {
    public constructor(observer: MouseObserver) {
        const onMouseMove = (event: MouseEvent) => {
            observer.on_move(event.clientX, event.clientY);
        };
        const onMouseDown = (event: MouseEvent) => {
            observer.on_down(event.button, event.clientX, event.clientY);
        };
        const onMouseUp = (event: MouseEvent) => {
            observer.on_up(event.button, event.clientX, event.clientY);
        };
        document.addEventListener("contextmenu", e => e.preventDefault());
        document.addEventListener("mousemove", onMouseMove);
        document.addEventListener("mousedown", onMouseDown);
        document.addEventListener("mouseup", onMouseUp);
        document.addEventListener("wheel", (event: WheelEvent) => {
            observer.on_scroll(event.deltaX, event.deltaY);
        });
    }
}