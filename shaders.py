# shaders.py

vertex_shader_3d = """
#version 330

// Attributs par sommet (le quad de base)
in vec2 in_vert;

// Attributs par Instance (données du rayon)
in float in_x_offset;    // Position X à l'écran (normalisée -1 à 1)
in float in_width;       // Largeur de la bande
in float in_height;      // Hauteur de la bande
in float in_distance;    // Distance pour le calcul HDR
in vec3  in_color;       // Couleur de base (r, g, b normalisés)

out float v_distance;
out vec3  v_color;

void main() {
    // Calcul de la position
    // in_vert.x va de 0 à 1. On scale par width, on décale par x_offset
    // in_vert.y va de 0 à 1 (ou -0.5 à 0.5 selon definition).

    // On centre verticalement
    vec2 pos = in_vert; 

    // Scaling vertical basé sur la hauteur du mur
    float y = (pos.y - 0.5) * in_height; 

    // Position X et largeur
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
    // --- LOGIQUE HDR ---
    // Au lieu de "dimmer" bêtement la couleur, on calcule une intensité physique.
    // L'intensité lumineuse décroît avec le carré de la distance.

    float dist = max(v_distance, 1.0); // Eviter division par zero

    // Formule d'atténuation (Inverse Square Law simulée)
    // Plus u_light_intensity est grand, plus la lumière porte loin.
    float decay = (dist / u_light_intensity);
    float brightness = 1.0 / (1.0 + decay * decay);

    // Clamp minimal pour ne pas avoir de noir total (lumière ambiante)
    brightness = max(brightness, u_min_brightness);

    // Application de la luminosité sur la couleur de base
    vec3 hdr_color = v_color * brightness;

    // Ici, nous pourrions appliquer un Tone Mapping si nous voulions simuler une caméra,
    // mais pour cet effet "néon/laser", le clamp naturel de la sortie écran (0.0-1.0)
    // donne un bon rendu saturé.

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
    // Si alpha est 0, on discard pour voir la 3D derrière (si on superpose)
    if (tex_color.a < 0.1) discard;
    f_color = tex_color;
}
"""