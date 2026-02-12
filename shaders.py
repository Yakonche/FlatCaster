# shaders.py

vertex_shader_3d = """
#version 330

in vec2 in_vert;

in float in_x_offset;    
in float in_width;       
in float in_height;      
in float in_distance;    
in vec3  in_color;       

out float v_distance;
out vec3  v_color;

void main() {
    vec2 pos = in_vert; 
    float y = (pos.y - 0.5) * in_height; 
    float x = in_x_offset + (pos.x * in_width);

    gl_Position = vec4(x, y, 0.0, 1.0);

    v_distance = in_distance;
    v_color = in_color;
}
"""

fragment_shader_3d = """
#version 330

in float v_distance;
in vec3  v_color;

out vec4 f_color;

uniform float u_light_intensity;
uniform float u_min_brightness;

void main() {
    float dist = max(v_distance, 1.0); 
    float decay = (dist / u_light_intensity);
    float brightness = 1.0 / (1.0 + decay * decay);
    brightness = max(brightness, u_min_brightness);

    vec3 hdr_color = v_color * brightness;
    f_color = vec4(hdr_color, 1.0);
}
"""

vertex_shader_2d = """
#version 330
in vec2 in_vert;
in vec2 in_texcoord;
out vec2 v_texcoord;

void main() {
    gl_Position = vec4(in_vert, 0.0, 1.0);
    v_texcoord = in_texcoord;
}
"""

fragment_shader_2d = """
#version 330
uniform sampler2D u_texture;
in vec2 v_texcoord;
out vec4 f_color;

void main() {
    vec4 tex_color = texture(u_texture, v_texcoord);
    if (tex_color.a < 0.1) discard;
    f_color = tex_color;
}
"""


vertex_shader_map_lines = """
#version 430
layout(std430, binding = 0) buffer MapData {
    float segmentData[];
};

uniform vec2 u_offset;
uniform float u_tile_size;
uniform vec2 u_resolution;

out vec3 v_color;

vec3 getWallColor(int wallType) {
    if (wallType == 1) return vec3(200.0/255.0, 50.0/255.0, 50.0/255.0);
    if (wallType == 2) return vec3(50.0/255.0, 200.0/255.0, 50.0/255.0);
    if (wallType == 3) return vec3(50.0/255.0, 50.0/255.0, 200.0/255.0);
    if (wallType == 4) return vec3(150.0/255.0, 50.0/255.0, 150.0/255.0);
    if (wallType == 5) return vec3(200.0/255.0, 100.0/255.0, 50.0/255.0);
    return vec3(1.0);
}

void main() {
    int segmentIdx = gl_VertexID / 2;
    int isEnd = gl_VertexID % 2;
    int baseIdx = segmentIdx * 5;

    float x = (isEnd == 0) ? segmentData[baseIdx] : segmentData[baseIdx+2];
    float y = (isEnd == 0) ? segmentData[baseIdx+1] : segmentData[baseIdx+3];
    int c = int(segmentData[baseIdx+4] + 0.1);

    vec2 screen_pos = vec2(x, y) * u_tile_size + u_offset;
    vec2 clip_pos = (screen_pos / u_resolution) * 2.0 - 1.0;
    clip_pos.y = -clip_pos.y; // Inversion Y (Pygame = Y descend, OpenGL = Y monte)

    gl_Position = vec4(clip_pos, 0.0, 1.0);
    v_color = getWallColor(c);
}
"""

fragment_shader_map_lines = """
#version 430
in vec3 v_color;
out vec4 f_color;
void main() {
    f_color = vec4(v_color, 1.0);
}
"""

vertex_shader_fov = """
#version 430
layout(std430, binding = 2) buffer HitBuffer {
    vec4 hitData[];
};

uniform vec2 u_player_pos;
uniform vec2 u_offset;
uniform float u_zoom;
uniform vec2 u_resolution;
uniform int u_step;
uniform int u_max_ray;

void main() {
    vec2 world_pos;
    if (gl_VertexID == 0) {
        world_pos = u_player_pos; // Centre du cône (le joueur)
    } else {
        int rayIdx = (gl_VertexID - 1) * u_step;
        if (rayIdx >= u_max_ray) {
            rayIdx = u_max_ray - 1;
        }
        world_pos = vec2(hitData[rayIdx].y, hitData[rayIdx].z); // Points d'impact
    }

    vec2 screen_pos = world_pos * u_zoom + u_offset;
    vec2 clip_pos = (screen_pos / u_resolution) * 2.0 - 1.0;
    clip_pos.y = -clip_pos.y;

    gl_Position = vec4(clip_pos, 0.0, 1.0);
}
"""

fragment_shader_fov = """
#version 430
out vec4 f_color;
void main() {
    f_color = vec4(200.0/255.0, 200.0/255.0, 200.0/255.0, 40.0/255.0);
}
"""