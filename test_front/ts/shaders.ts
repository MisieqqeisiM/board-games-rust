export const VERTEX_SHADER_SOURCE = `#version 300 es
    in vec2 a_position;
    in vec2 a_texCoord;
    in float a_atlas;
    uniform mat3 u_transform;
    uniform vec2 u_aspect;
    out vec2 v_texCoord;
    out float v_atlas;
    void main() {
        vec3 pos = u_transform * vec3(a_position, 1);
        pos.y *= -1.0;
        pos.xy *= 2.0 / u_aspect;
        pos.xy += vec2(-1.0, 1.0);

        gl_Position = vec4(pos.x, pos.y, 0, 1);
        v_texCoord = a_texCoord;
        v_atlas = a_atlas;
    }
`;

export const FRAGMENT_SHADER_SOURCE = `#version 300 es
    precision mediump float;
    in vec2 v_texCoord;
    in float v_atlas;
    out vec4 outColor;
    uniform sampler2D tex0;
    uniform sampler2D tex1;
    uniform sampler2D tex2;
    uniform sampler2D tex3;
    void main() {
        if(v_atlas == 0.0) {
            outColor = texture(tex0, v_texCoord);
        } else if(v_atlas == 1.0) {
            outColor = texture(tex1, v_texCoord);
        } else if(v_atlas == 2.0) {
            outColor = texture(tex2, v_texCoord);
        } else if(v_atlas == 3.0) {
            outColor = texture(tex3, v_texCoord);
        } else {
            outColor = vec4(0, 0, 0, 1);
        }
    }
`;

export function createShader(gl: WebGL2RenderingContext, type: GLenum, source: string) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    const success = gl.getShaderParameter(shader, gl.COMPILE_STATUS);
    if (success) {
        return shader;
    }

    console.log(gl.getShaderInfoLog(shader));
    gl.deleteShader(shader);
}


export function createProgram(gl: WebGL2RenderingContext, vertexShader: WebGLShader, fragmentShader: WebGLShader) {
    const program = gl.createProgram();
    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);
    const success = gl.getProgramParameter(program, gl.LINK_STATUS);
    if (success) {
        return program;
    }

    console.log(gl.getProgramInfoLog(program));
    gl.deleteProgram(program);
}

export function init(gl: WebGL2RenderingContext) {


}
