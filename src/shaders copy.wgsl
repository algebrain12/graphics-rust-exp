alias TriangleVertices = array<vec2f, 3>;
var<private> vertices: TriangleVertices = TriangleVertices(
  vec2f(-1.0, -1.0),
  vec2f( 3.0, -1.0),
  vec2f( -1.0,  3.0),
);

struct Sphere{
  radius: f32,
  position: vec3f,
  material: Material
}
struct Ray{
  origin: vec3f,
  dir: vec3f
}

struct Uniforms {
  width: u32,
  height: u32,
} 
@group(0) @binding(0) var<uniform> uniforms: Uniforms;




@vertex fn path_tracer_vs(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4f {
  return vec4f(vertices[vid], 0.0, 1.0);
}

@fragment fn path_tracer_fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let x = (pos.x/f32(uniforms.width) - 0.5)*2* f32(uniforms.width) / f32(uniforms.height);
    let y = -(pos.y/f32(uniforms.height) - 0.5)*2;
    var spheres:array<Sphere, 3> = array<Sphere,3>(Sphere(0.2,  vec3(0,0,-1), Material(vec4(1.0), 0.0)), Sphere(0.2,  vec3(0,0,-1), Material(vec4(1.0), 0.0)), Sphere(0.2,  vec3(0,0,-1), Material(vec4(1.0), 0.0)));


    return vec4<f32>(1 - (y+1)/2, 1 - (y+1)/2, 1, 1);
}

struct HitInfo{
  hit:bool,
  normal:vec3<f32>,
  pos:vec3<f32>,
  time:f32
}

struct Material{
  color:vec4<f32>,
  reflections:f32,
}

fn Hit_sphere(sphere:Sphere, ray:Ray) -> HitInfo{
  let a:f32 = dot(sphere.position - ray.origin, ray.dir) / abs(ray.dir);
  let b:f32 = abs(sphere.position - ray.origin);
  let final1 = get_pos(ray.origin, ray.dir, sphere.position, sphere.radius);
  if(b*b - a*a < sphere.radius*sphere.radius){
    let normal = (final1-sphere.position)/abs(final1-sphere.position);
    return HitInfo(true, normal, final1, abs(final1-ray.origin));
  }
  return HitInfo(false, vec3(0,0,0), vec3(0,0,0), 0);
}

fn abs(vector:vec3f) -> f32{
  return pow(vector.x*vector.x + vector.y*vector.y + vector.z*vector.z, 0.5);
}

fn dot(vector1:vec3f, vector2:vec3f) -> f32{
  return vector1.x*vector2.x + vector1.y*vector2.y + vector1.z*vector2.z;
}

fn get_pos(
    origin: vec3<f32>,
    dis: vec3<f32>,
    center: vec3<f32>,
    radius: f32
) -> vec3<f32> {
    let dir = dis / abs(dis);
    let oc = origin - center;
    let a = dot(dir, dir);
    let b = 2.0 * dot(oc, dir);
    let c = dot(oc, oc) - radius * radius;

    let discriminant = b*b - 4.0*a*c;

    if (discriminant < 0.0) {
        // No hit: return some sentinel value
        return vec3<f32>(9999.0, 9999.0, 9999.0);
    }

    // smallest positive root
    let t = (-b - sqrt(discriminant)) / (2.0 * a);
    return origin + t * dir;
}