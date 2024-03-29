pub const BACKGROUND_VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core

in vec2 position;
in vec4 color;

out vec4 v_color;

uniform mat4 projection;

void main()
{
    v_color = color;
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

#[allow(dead_code)]
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
    const vec3 blur[9] = vec3[](
        vec3(-1.0,  1.0, 1.0 / 16.0),
        vec3(-1.0,  0.0, 2.0 / 16.0),
        vec3(-1.0,  1.0, 1.0 / 16.0),
        vec3( 0.0, -1.0, 2.0 / 16.0),
        vec3( 0.0,  0.0, 4.0 / 16.0),
        vec3( 0.0,  1.0, 2.0 / 16.0),
        vec3( 1.0, -1.0, 1.0 / 16.0),
        vec3( 1.0,  0.0, 2.0 / 16.0),
        vec3( 1.0,  1.0, 1.0 / 16.0)
    );

    vec2 texel;
    vec4 modifier = vec4(0.0);

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

pub const FOREGROUND_FRAGMENT_SHADER_SDF_SOURCE: &str = r#"
#version 330 core

#define SMOOTHING 0.09
#define BUFFER 0.475

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

vec4 calculate_frag_color(float distance) {
    float alpha = smoothstep(BUFFER - SMOOTHING, BUFFER + SMOOTHING, distance);
    vec4 frag_color = vec4(v_color.rgb, 1.0) * alpha * v_color.a;
    frag_color.a += frag_color.a * 0.3;
    return frag_color;
}

void main()
{
    vec4 frag_color;
    float distance, alpha;

    // In GLSL 330 non-const values cannot be used for indexing arrays in fragment shaders.
    int index = int(floor(v_tex_index));

    switch(index) {
    case 0:
        frag_color = calculate_frag_color(texture2D(regular, v_tex_coords).a);
        break;
    case 1:
        frag_color = calculate_frag_color(texture2D(bold, v_tex_coords).a);
        break;
    case 2:
        frag_color = calculate_frag_color(texture2D(italic, v_tex_coords).a);
        break;
    case 3:
        frag_color = calculate_frag_color(texture2D(bold_italic, v_tex_coords).a);
        break;
    case 4:
        frag_color = calculate_frag_color(texture2D(regular_outline, v_tex_coords).a);
        break;
    case 5:
        frag_color = calculate_frag_color(texture2D(bold_outline, v_tex_coords).a);
        break;
    case 6:
        frag_color = calculate_frag_color(texture2D(italic_outline, v_tex_coords).a);
        break;
    case 7:
        frag_color = calculate_frag_color(texture2D(bold_italic_outline, v_tex_coords).a);
        break;
    }

    color = frag_color;
}
"#;

pub const FULL_FRAME_VERTEX_SHADER_SOURCE: &str = r#"
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

precision highp float;

in vec2 v_coords;

out vec4 color;

// Adapted from https://shader-tutorial.dev/advanced/color-banding-dithering/
float random(vec2 coords) {
   return fract(sin(dot(coords.xy, vec2(12.9898,78.233))) * 43758.5453);
}

void main()
{
    // Invert the coords so that the center is brigher.
    vec2 coords = v_coords;
    coords *= 1.0 - v_coords.yx;

    // The multiplicand literal determines the inner radius of the vignette.
    float vignette = coords.x * coords.y * 20.0;

    // The exponent determines the intensity of the vignette.
    vignette = pow(vignette, 0.15);

    color = vec4(0.0, 0.0, 0.0, 1.0 - vignette);

    // Determines the noise level. Less than 5.0 results in noticeable banding.
    const float granularity = 5.0 / 255.0;
    color.a += mix(-granularity, granularity, color.a + random(coords));
}
"#;
