pub const BACKGROUND_VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core

in vec2 position;
in vec3 color;

out vec4 v_color;

uniform mat4 mvp;

void main()
{
    v_color = vec4(color, 1.0);
    gl_Position = mvp * vec4(position, 1.0, 1.0);
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

out vec4 v_color;
out vec2 v_tex_coords;

uniform mat4 mvp;

void main()
{
    v_color = color;
    v_tex_coords = tex_coords;
    gl_Position = mvp * vec4(position, 1.0, 1.0);
}
"#;

pub const FOREGROUND_FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core

precision lowp float;

in vec4 v_color;
in vec2 v_tex_coords;

out vec4 color;

uniform sampler2D texture;

void main()
{
    // Apply a subtle blur to reduce scaling artifacts.
    // TODO: Render to framebuffer instead and use better shading techniques?
    vec3 blur[9];
    blur[0] = vec3(-1.0,  1.0, 1.0 / 16.0);
    blur[1] = vec3(-1.0,  0.0, 2.0 / 16.0);
    blur[2] = vec3(-1.0,  1.0, 1.0 / 16.0);
    blur[3] = vec3( 0.0, -1.0, 2.0 / 16.0);
    blur[4] = vec3( 0.0,  0.0, 4.0 / 16.0);
    blur[5] = vec3( 0.0,  1.0, 2.0 / 16.0);
    blur[6] = vec3( 1.0, -1.0, 1.0 / 16.0);
    blur[7] = vec3( 1.0,  0.0, 2.0 / 16.0);
    blur[8] = vec3( 1.0,  1.0, 1.0 / 16.0);
    vec2 texel = vec2(1.0) / textureSize(texture, 0);
    vec4 modifier = vec4(0.0);

    for (int i = 0; i < blur.length(); ++i) {
        modifier += blur[i].z * texture2D(texture, v_tex_coords + texel * blur[i].xy);
    }

    color = v_color * modifier;
}
"#;
