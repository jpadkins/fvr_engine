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
    color = texture2D(texture, v_tex_coords) * v_color;
}
"#;
