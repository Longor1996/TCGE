#version 330 core
in vec3 position;
out vec4 Color;

uniform float time = 0;

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

vec3 hash3( vec2 p ) {
    vec3 q = vec3( dot(p,vec2(127.1,311.7)),
    dot(p,vec2(269.5,183.3)),
    dot(p,vec2(419.2,371.9)) );
    return fract(sin(q)*43758.5453);
}

float iqnoise( in vec2 x, float u, float v ) {
    vec2 p = floor(x);
    vec2 f = fract(x);
    float k = 1.0+63.0*pow(1.0-v,4.0);
    float va = 0.0;
    float wt = 0.0;

    for( int j=-2; j<=2; j++ )
    for( int i=-2; i<=2; i++ )
    {
        vec2 g = vec2( float(i),float(j) );
        vec3 o = hash3( p + g )*vec3(u,u,1.0);
        vec2 r = g - f + o.xy;
        float d = dot(r,r);
        float ww = pow( 1.0-smoothstep(0.0,1.414,sqrt(d)), k );
        va += o.z*ww;
        wt += ww;
    }

    return va/wt;
}

void main() {
    float n_xy = iqnoise(outPosition.xy*1.0, 1.0, 0.0);
    float n_xz = iqnoise(outPosition.xz*1.0, 1.0, 0.0);
    float n_yz = iqnoise(outPosition.yz*1.0, 1.0, 0.0);

    Color = vec4(n_xy,n_xz,n_yz,1.0);
}