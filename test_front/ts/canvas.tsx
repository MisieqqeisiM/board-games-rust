import { WebGLFloatVector } from "./glvector";
import {
    createProgram, createShader,
    FRAGMENT_SHADER_SOURCE, VERTEX_SHADER_SOURCE,
    LINE_VERTEX_SHADER_SOURCE, LINE_FRAGMENT_SHADER_SOURCE
} from "./shaders";

export class Canvas {
    private element: HTMLCanvasElement;
    private gl: WebGL2RenderingContext;

    // image program
    private imageProgram: WebGLProgram;
    private transformLocation: WebGLUniformLocation;
    private aspectLocation: WebGLUniformLocation;
    private positionAttributeLocation: number;
    private texCoordAttributeLocation: number;
    private atlasCoordAttributeLocation: number;
    private image_groups: WebGLFloatVector[];
    private atlases: WebGLTexture[] = [];

    // line program
    private lineProgram: WebGLProgram;
    private lineTransformLocation: WebGLUniformLocation;
    private lineAspectLocation: WebGLUniformLocation;
    private linePosAttr: number;
    private lineColorAttr: number;
    private lines: WebGLFloatVector;

    // CPU-side transform state so we can push to both programs
    private tx = 0; private ty = 0; private tscale = 1;

    constructor() {
        this.element = document.createElement("canvas");
        this.element.style.width = "100%";
        this.element.style.height = "100%";
        this.gl = this.element.getContext("webgl2");
        window.onresize = () => { this.fixAspect(); this.draw(); };

        // ── image program ──────────────────────────────────────────
        const vs = createShader(this.gl, this.gl.VERTEX_SHADER, VERTEX_SHADER_SOURCE);
        const fs = createShader(this.gl, this.gl.FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);
        this.imageProgram = createProgram(this.gl, vs, fs);
        this.gl.useProgram(this.imageProgram);

        this.positionAttributeLocation = this.gl.getAttribLocation(this.imageProgram, "a_position");
        this.texCoordAttributeLocation = this.gl.getAttribLocation(this.imageProgram, "a_texCoord");
        this.atlasCoordAttributeLocation = this.gl.getAttribLocation(this.imageProgram, "a_atlas");
        this.image_groups = [];

        const vao = this.gl.createVertexArray();
        this.gl.bindVertexArray(vao);

        this.transformLocation = this.gl.getUniformLocation(this.imageProgram, "u_transform");
        this.aspectLocation = this.gl.getUniformLocation(this.imageProgram, "u_aspect");

        for (let i = 0; i < 8; i++) {
            this.gl.uniform1i(this.gl.getUniformLocation(this.imageProgram, `tex${i}`), i);
        }

        // ── line program ───────────────────────────────────────────
        const lvs = createShader(this.gl, this.gl.VERTEX_SHADER, LINE_VERTEX_SHADER_SOURCE);
        const lfs = createShader(this.gl, this.gl.FRAGMENT_SHADER, LINE_FRAGMENT_SHADER_SOURCE);
        this.lines = new WebGLFloatVector(this.gl, 1024, this.gl.STATIC_DRAW);
        this.lineProgram = createProgram(this.gl, lvs, lfs);

        this.lineTransformLocation = this.gl.getUniformLocation(this.lineProgram, "u_transform");
        this.lineAspectLocation = this.gl.getUniformLocation(this.lineProgram, "u_aspect");
        this.linePosAttr = this.gl.getAttribLocation(this.lineProgram, "a_position");
        this.lineColorAttr = this.gl.getAttribLocation(this.lineProgram, "a_color");

        document.body.appendChild(this.element);
        this.fixAspect();
        this.setTransform(0, 0, 1);
    }

    // ── public API ─────────────────────────────────────────────────


    pushLine(vertices: Float32Array): void {
        this.lines.push(vertices);
    }


    fixAspect() {
        this.element.width = this.element.clientWidth;
        this.element.height = this.element.clientHeight;
        this.gl.viewport(0, 0, this.element.width, this.element.height);
        const aspect = new Float32Array([this.element.width, this.element.height]);
        this.gl.useProgram(this.imageProgram);
        this.gl.uniform2fv(this.aspectLocation, aspect);
        this.gl.useProgram(this.lineProgram);
        this.gl.uniform2fv(this.lineAspectLocation, aspect);
    }

    setTransform(x: number, y: number, scale: number) {
        this.tx = x; this.ty = y; this.tscale = scale;
        const m = new Float32Array([scale, 0, 0, 0, scale, 0, -x * scale, -y * scale, 1]);
        this.gl.useProgram(this.imageProgram);
        this.gl.uniformMatrix3fv(this.transformLocation, false, m);
        this.gl.useProgram(this.lineProgram);
        this.gl.uniformMatrix3fv(this.lineTransformLocation, false, m);
    }

    // ── atlas / image API (unchanged) ──────────────────────────────

    createAtlas() {
        if (this.atlases.length % 8 === 0)
            this.image_groups.push(new WebGLFloatVector(this.gl, 1024, this.gl.STATIC_DRAW));
        const atlas = this.gl.createTexture();
        this.gl.bindTexture(this.gl.TEXTURE_2D, atlas);
        this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_S, this.gl.CLAMP_TO_EDGE);
        this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_T, this.gl.CLAMP_TO_EDGE);
        this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MIN_FILTER, this.gl.NEAREST);
        this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MAG_FILTER, this.gl.NEAREST);
        this.gl.texImage2D(this.gl.TEXTURE_2D, 0, this.gl.RGBA, 2048, 2048, 0, this.gl.RGBA, this.gl.UNSIGNED_BYTE, null);
        this.atlases.push(atlas);
    }

    updateAtlas(data: Uint8Array, atlas_id: number, x: number, y: number, width: number, height: number) {
        this.gl.bindTexture(this.gl.TEXTURE_2D, this.atlases[atlas_id]);
        this.gl.texSubImage2D(this.gl.TEXTURE_2D, 0, x, y, width, height, this.gl.RGBA, this.gl.UNSIGNED_BYTE, data);
    }

    push(group: number, vertices: Float32Array) {
        this.image_groups[group].push(new Float32Array(vertices));
    }

    bindTextures(group: number) {
        for (let i = 0; i < 8; i++) {
            this.gl.activeTexture(this.gl.TEXTURE0 + i);
            this.gl.bindTexture(this.gl.TEXTURE_2D, this.atlases[group * 8 + i]);
        }
    }

    // ── draw ───────────────────────────────────────────────────────

    draw() {
        this.gl.clearColor(0, 0, 0, 0);
        this.gl.clear(this.gl.COLOR_BUFFER_BIT);
        this.drawImages();
        this.drawLines();
    }

    private drawImages() {
        this.gl.useProgram(this.imageProgram);
        for (let g = 0; g < this.image_groups.length; g++) {
            const vec = this.image_groups[g];
            this.gl.bindBuffer(this.gl.ARRAY_BUFFER, vec.getBuffer());
            this.gl.enableVertexAttribArray(this.positionAttributeLocation);
            this.gl.enableVertexAttribArray(this.texCoordAttributeLocation);
            this.gl.enableVertexAttribArray(this.atlasCoordAttributeLocation);
            this.gl.vertexAttribPointer(this.positionAttributeLocation, 2, this.gl.FLOAT, false, 5 * 4, 0);
            this.gl.vertexAttribPointer(this.texCoordAttributeLocation, 2, this.gl.FLOAT, false, 5 * 4, 2 * 4);
            this.gl.vertexAttribPointer(this.atlasCoordAttributeLocation, 1, this.gl.FLOAT, false, 5 * 4, 4 * 4);
            this.bindTextures(g);
            this.gl.drawArrays(this.gl.TRIANGLES, 0, vec.size() / 5);
        }
    }
    private drawLines() {
        this.gl.useProgram(this.lineProgram);

        this.gl.enable(this.gl.BLEND);
        this.gl.blendFunc(this.gl.SRC_ALPHA, this.gl.ONE_MINUS_SRC_ALPHA);

        const vec = this.lines;
        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, vec.getBuffer());
        this.gl.enableVertexAttribArray(this.linePosAttr);
        this.gl.enableVertexAttribArray(this.lineColorAttr);

        this.gl.vertexAttribPointer(this.linePosAttr,   2, this.gl.FLOAT, false, 6 * 4, 0);
        this.gl.vertexAttribPointer(this.lineColorAttr, 4, this.gl.FLOAT, false, 6 * 4, 2 * 4);
        this.gl.drawArrays(this.gl.TRIANGLE_STRIP, 0, vec.size() / 6);
    }
}