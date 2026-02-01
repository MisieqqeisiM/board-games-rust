export class WebGLFloatVector {
    private gl: WebGLRenderingContext | WebGL2RenderingContext
    private buffer: WebGLBuffer
    private data: Float32Array

    private length: number
    private capacity: number

    constructor(
        gl: WebGLRenderingContext | WebGL2RenderingContext,
        initialCapacity: number = 1024,
        usage: number = gl.DYNAMIC_DRAW
    ) {
        this.gl = gl
        this.capacity = initialCapacity
        this.length = 0
        this.data = new Float32Array(this.capacity)

        const buffer = gl.createBuffer()
        if (!buffer) {
            throw new Error("Failed to create WebGL buffer")
        }
        this.buffer = buffer

        gl.bindBuffer(gl.ARRAY_BUFFER, this.buffer)
        gl.bufferData(
            gl.ARRAY_BUFFER,
            this.data.byteLength,
            usage
        )
    }

    /**
     * Pushes floats into the vector.
     * Grows capacity x2 if needed.
     */
    push(vertices: Float32Array): void {
        const required = this.length + vertices.length

        if (required > this.capacity) {
            this.grow(required)
        }

        this.data.set(vertices, this.length)
        this.length += vertices.length

        // Upload only the new portion
        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffer)
        this.gl.bufferSubData(
            this.gl.ARRAY_BUFFER,
            (this.length - vertices.length) * 4,
            vertices
        )
    }

    size(): number {
        return this.length
    }

    getBuffer(): WebGLBuffer {
        return this.buffer
    }

    private grow(minCapacity: number): void {
        while (this.capacity < minCapacity) {
            this.capacity *= 2
        }

        const newData = new Float32Array(this.capacity)
        newData.set(this.data)
        this.data = newData

        // Reallocate GPU buffer
        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.buffer)
        this.gl.bufferData(
            this.gl.ARRAY_BUFFER,
            this.data.byteLength,
            this.gl.DYNAMIC_DRAW
        )

        // Re-upload existing data
        this.gl.bufferSubData(
            this.gl.ARRAY_BUFFER,
            0,
            this.data.subarray(0, this.length)
        )
    }
}
