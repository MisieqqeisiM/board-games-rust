export interface PasteObserver {
    on_file(data: Uint8Array): void;
}

export class Paste {
    public constructor(observer: PasteObserver, path: String) {
        document.addEventListener("paste", async e => {
            const clipboardData = (e as ClipboardEvent).clipboardData;
            if (!clipboardData) {
                return;
            }
            const items = clipboardData.items;
            for (let i = 0; i < items.length; i++) {
                const item = items[i];
                if (item.kind === "file" && item.type === "image/png") {
                    const file = item.getAsFile();
                    if (file) {
                        const arrayBuffer = await file.arrayBuffer();
                        observer.on_file(new Uint8Array(arrayBuffer));
                    }
                }
            }
        });
    }
}