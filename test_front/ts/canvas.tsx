import { WebGLFloatVector } from "./glvector";
import { createProgram, createShader, FRAGMENT_SHADER_SOURCE, init, VERTEX_SHADER_SOURCE } from "./shaders";

export class Canvas {
    private element: HTMLCanvasElement;
    private transformLocation: WebGLUniformLocation;
    private aspectLocation: WebGLUniformLocation;
    private vertices: WebGLFloatVector;
    private gl: WebGL2RenderingContext;
    private positionAttributeLocation: number;
    private texCoordAttributeLocation: number
    private atlasCoordAttributeLocation: number;
    private atlases: WebGLTexture[] = [];

    constructor() {
        this.element = document.createElement("canvas");
        this.element.style.width = "100%";
        this.element.style.height = "100%";
        this.gl = this.element.getContext("webgl2");
        window.onresize = () => {
            this.fixAspect();
            this.draw();
        };
        for (let i = 0; i < 4; i++) {
            const atlas = this.gl.createTexture();
            this.gl.bindTexture(this.gl.TEXTURE_2D, atlas);
            this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_S, this.gl.CLAMP_TO_EDGE);
            this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_T, this.gl.CLAMP_TO_EDGE);
            this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MIN_FILTER, this.gl.NEAREST);
            this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MAG_FILTER, this.gl.NEAREST);
            this.gl.texImage2D(this.gl.TEXTURE_2D, 0, this.gl.RGBA, 2048, 2048, 0, this.gl.RGBA, this.gl.UNSIGNED_BYTE, null);
            this.gl.activeTexture(this.gl.TEXTURE0 + i);
            this.gl.bindTexture(this.gl.TEXTURE_2D, atlas);
            this.atlases.push(atlas);
        }
        const vertexShader = createShader(this.gl, this.gl.VERTEX_SHADER, VERTEX_SHADER_SOURCE);
        const fragmentShader = createShader(this.gl, this.gl.FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);
        const program = createProgram(this.gl, vertexShader, fragmentShader);
        this.gl.useProgram(program);
        this.positionAttributeLocation = this.gl.getAttribLocation(program, "a_position");
        this.texCoordAttributeLocation = this.gl.getAttribLocation(program, "a_texCoord");
        this.atlasCoordAttributeLocation = this.gl.getAttribLocation(program, "a_atlas");
        this.vertices = new WebGLFloatVector(this.gl, 1024, this.gl.STATIC_DRAW);
        const vao = this.gl.createVertexArray();
        this.gl.bindVertexArray(vao);
        this.transformLocation = this.gl.getUniformLocation(this.gl.getParameter(this.gl.CURRENT_PROGRAM), "u_transform");
        this.aspectLocation = this.gl.getUniformLocation(this.gl.getParameter(this.gl.CURRENT_PROGRAM), "u_aspect");
        this.setTransform(0.0, 0.0, 1);
        const tex0UniformLocation = this.gl.getUniformLocation(program, "tex0");
        const tex1UniformLocation = this.gl.getUniformLocation(program, "tex1");
        const tex2UniformLocation = this.gl.getUniformLocation(program, "tex2");
        const tex3UniformLocation = this.gl.getUniformLocation(program, "tex3");
        this.gl.uniform1i(tex0UniformLocation, 0);
        this.gl.uniform1i(tex1UniformLocation, 1);
        this.gl.uniform1i(tex2UniformLocation, 2);
        this.gl.uniform1i(tex3UniformLocation, 3);

        document.body.appendChild(this.element);
        this.fixAspect();
    }

    fixAspect() {
        this.element.width = this.element.clientWidth;
        this.element.height = this.element.clientHeight;
        this.gl.viewport(0, 0, this.element.width, this.element.height);
        this.gl.uniform2fv(this.aspectLocation, new Float32Array([this.element.width, this.element.height]));
    }

    updateAtlas(data: Uint8Array, atlas_id: number, x: number, y: number, width: number, height: number) {
        const gl = this.element.getContext("webgl2");
        gl.activeTexture(gl.TEXTURE0 + atlas_id);
        gl.bindTexture(gl.TEXTURE_2D, this.atlases[atlas_id]);
        gl.texSubImage2D(gl.TEXTURE_2D, 0, x, y, width, height, gl.RGBA, gl.UNSIGNED_BYTE, data);
    }

    setTransform(x: number, y: number, scale: number) {
        const gl = this.element.getContext("webgl2");
        gl.uniformMatrix3fv(this.transformLocation, false, new Float32Array([
            scale, 0, 0,
            0, scale, 0,
            -x * scale, -y * scale, 1
        ]));
    }

    push(vertices: Float32Array) {
        this.vertices.push(new Float32Array(vertices));
        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.vertices.getBuffer());
        this.gl.enableVertexAttribArray(this.positionAttributeLocation);
        this.gl.enableVertexAttribArray(this.texCoordAttributeLocation);
        this.gl.enableVertexAttribArray(this.atlasCoordAttributeLocation);
        this.gl.vertexAttribPointer(this.positionAttributeLocation, 2, this.gl.FLOAT, false, 5 * 4, 0);
        this.gl.vertexAttribPointer(this.texCoordAttributeLocation, 2, this.gl.FLOAT, false, 5 * 4, 2 * 4);
        this.gl.vertexAttribPointer(this.atlasCoordAttributeLocation, 1, this.gl.FLOAT, false, 5 * 4, 4 * 4);
    }


    draw() {
        const gl = this.element.getContext("webgl2");
        gl.clearColor(0, 0, 0, 0);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.drawArrays(gl.TRIANGLES, 0, this.vertices.size() / 5);
    }

}