pub const BACKGROUND_VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core

in vec2 position;
in vec3 color;

out vec4 v_color;

uniform mat4 projection;

void main()
{
    v_color = vec4(color, 1.0);
    gl_Position = projection * vec4(position, 1.0, 1.0);
}
"#;

pub const BACKGROUND_FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core

precision lowp float;

in vec4 v_color;

out vec4 color;

void main()
{
    color = v_color;
}
"#;

pub const FOREGROUND_VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core

in vec2 position;
in vec4 color;
in vec2 tex_coords;
in float tex_index;

out vec4 v_color;
out vec2 v_tex_coords;
out float v_tex_index;

uniform mat4 projection;

void main()
{
    v_color = color;
    v_tex_coords = tex_coords;
    v_tex_index = tex_index;
    gl_Position = projection * vec4(position, 1.0, 1.0);
}
"#;

pub const FOREGROUND_FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core

precision highp float;

in vec4 v_color;
in vec2 v_tex_coords;
in float v_tex_index;

out vec4 color;

uniform sampler2D regular;
uniform sampler2D bold;
uniform sampler2D italic;
uniform sampler2D bold_italic;
uniform sampler2D regular_outline;
uniform sampler2D bold_outline;
uniform sampler2D italic_outline;
uniform sampler2D bold_italic_outline;

void main()
{
    // Apply a subtle blur to reduce scaling artifacts.
    // TODO: Render to framebuffer instead and use better shading techniques?

    vec2 texel;
    vec3 blur[9];
    vec4 modifier = vec4(0.0);

    blur[0] = vec3(-1.0,  1.0, 1.0 / 16.0);
    blur[1] = vec3(-1.0,  0.0, 2.0 / 16.0);
    blur[2] = vec3(-1.0,  1.0, 1.0 / 16.0);
    blur[3] = vec3( 0.0, -1.0, 2.0 / 16.0);
    blur[4] = vec3( 0.0,  0.0, 4.0 / 16.0);
    blur[5] = vec3( 0.0,  1.0, 2.0 / 16.0);
    blur[6] = vec3( 1.0, -1.0, 1.0 / 16.0);
    blur[7] = vec3( 1.0,  0.0, 2.0 / 16.0);
    blur[8] = vec3( 1.0,  1.0, 1.0 / 16.0);

    // In GLSL 330 non-const values cannot be used for indexing arrays in fragment shaders.
    // Hence this garbage...

    int index = int(floor(v_tex_index));

    switch(index) {
    case 0:
        texel = vec2(1.0) / textureSize(regular, 0);

        for (int i = 0; i < blur.length(); ++i) {
            modifier += blur[i].z * texture2D(regular, v_tex_coords + texel * blur[i].xy);
        }

        break;
    case 1:
        texel = vec2(1.0) / textureSize(bold, 0);

        for (int i = 0; i < blur.length(); ++i) {
            modifier += blur[i].z * texture2D(bold, v_tex_coords + texel * blur[i].xy);
        }

        break;
    case 2:
        texel = vec2(1.0) / textureSize(italic, 0);

        for (int i = 0; i < blur.length(); ++i) {
            modifier += blur[i].z * texture2D(italic, v_tex_coords + texel * blur[i].xy);
        }

        break;
    case 3:
        texel = vec2(1.0) / textureSize(bold_italic, 0);

        for (int i = 0; i < blur.length(); ++i) {
            modifier += blur[i].z * texture2D(bold_italic, v_tex_coords + texel * blur[i].xy);
        }

        break;
    case 4:
        texel = vec2(1.0) / textureSize(regular_outline, 0);

        for (int i = 0; i < blur.length(); ++i) {
            modifier += blur[i].z * texture2D(regular_outline, v_tex_coords + texel * blur[i].xy);
        }

        break;
    case 5:
        texel = vec2(1.0) / textureSize(bold_outline, 0);

        for (int i = 0; i < blur.length(); ++i) {
            modifier += blur[i].z * texture2D(bold_outline, v_tex_coords + texel * blur[i].xy);
        }

        break;
    case 6:
        texel = vec2(1.0) / textureSize(italic_outline, 0);

        for (int i = 0; i < blur.length(); ++i) {
            modifier += blur[i].z * texture2D(italic_outline, v_tex_coords + texel * blur[i].xy);
        }

        break;
    case 7:
        texel = vec2(1.0) / textureSize(bold_italic_outline, 0);

        for (int i = 0; i < blur.length(); ++i) {
            modifier += blur[i].z * texture2D(bold_italic_outline, v_tex_coords + texel * blur[i].xy);
        }

        break;
    }

    color = v_color * modifier;
}
"#;

pub const VIGNETTE_VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core

out vec2 v_coords;

void main()
{
    const vec2 positions[4] = vec2[](
        vec2(-1, -1),
        vec2( 1, -1),
        vec2(-1,  1),
        vec2( 1,  1)
    );
    const vec2 coords[4] = vec2[](
        vec2(0, 0),
        vec2(1, 0),
        vec2(0, 1),
        vec2(1, 1)
    );

    v_coords = coords[gl_VertexID];
    gl_Position = vec4(positions[gl_VertexID], 0.0, 1.0);
}
"#;

pub const VIGNETTE_FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core

precision lowp float;

in vec2 v_coords;

out vec4 color;

void main()
{
    vec2 coords = v_coords;
    coords *= 1.0 - v_coords.yx;

    // The multiplicand determines the dimness of the entire frame.
    // Lower values increase dimness.
    float vignette = coords.x * coords.y * 15.0;

    // The exponent determines the intensity of the vignette.
    vignette = pow(vignette, 0.25);

    color = vec4(0.0, 0.0, 0.0, 1.0 - vignette);
}
"#;
