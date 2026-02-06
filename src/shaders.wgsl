alias TriangleVertices = array<vec2f, 3>;
var<private> vertices: TriangleVertices = TriangleVertices(
  vec2f(-1.0, -1.0),
  vec2f( 3.0, -1.0),
  vec2f( -1.0,  3.0),
);

const lambertian:f32 = 0.0;


var<private> length:i32 = 8;
var<private> spheres: array<Sphere, 8> = array<Sphere, 8>
(Sphere(0.05,  vec3(-0.05,0.02,-2.7), Material(vec4(0.9, 0.0, 0.9, 1), lambertian, vec3<f32>(0), 0.0, 0.0)), 
Sphere(0.04,  vec3(0.1,0.03,-2.4), Material(vec4(0.9, 0.9, 0.9, 1), 1.0, vec3<f32>(0), 0, 0)), 
Sphere(1,  vec3(0,-1,-3), Material(vec4(0.1, 0.9, 0.1, 1.0), lambertian, vec3<f32>(0,0,0), 0, 0)),
Sphere(0.05,  vec3(0.0,0.02,-2.5), Material(vec4(1.0), 1.0, vec3<f32>(0), 0.6, 1.6)),
Sphere(0.04,  vec3(-0.05,0.07,-2.6), Material(vec4(0.9, 0.9, 0.0, 1), lambertian+0.2, vec3<f32>(0), 0, 0)),
Sphere(0.04,  vec3(0.05,0.07,-2.3), Material(vec4(0.9, 0.0, 0.9, 1), lambertian, vec3<f32>(0), 0.0, 0.00)),
Sphere(0.1,  vec3(-0.3,0.09,-2.5), Material(vec4(0,1,1, 1), lambertian, 0*vec3<f32>(1), 0, 0)),
Sphere(0.07,  vec3(0.3,0.11,-2.6), Material(vec4(1,1,0, 1), lambertian, 0*vec3<f32>(1), 0, 0)));

var<private> sunDir:vec3<f32> = vec3<f32>(1,1,1);

var<private> MaxBounces:i32 = 100;

var<private> MaxTransBounces = 1000;

var<private> FOV:f32 = 1;

var<private> Samples:i32 = 1;

var<private> focus_distance = 1.0;

var<private> antiAliasing = false;

var<private> level = 1.0;

var<private> times: i32 = 10;


fn rand(p: vec3<f32>) -> f32 {
    let h = sin(dot(p, vec3<f32>(127.1, 311.7, 191.91))) * 43758.5453;
    return fract(h);
}
fn random_direction(seed: vec3<f32>) -> vec3<f32> {
    let u1 = rand(seed);
    let u2 = rand(seed + vec3<f32>(17.0, 31.0, 47.0));

    let theta = 2.0 * 3.14159265 * u1;  
    let z = 2.0 * u2 - 1.0;              
    let r = sqrt(max(0.0, 1.0 - z*z));
    
    return vec3<f32>(r * cos(theta), r * sin(theta), z);
}


struct Sphere{
  radius: f32,
  position: vec3f,
  material: Material
}
struct Ray{
  origin: vec3f,
  dir: vec3f
}

struct CameraUniforms {
    origin: vec3<f32>,
};


struct Uniforms {
  width: u32,
  height: u32,
  time:f32,
  _pad:f32,
  u:vec4f,
  v:vec4f,
  w:vec4f,
  ww:vec4f,
}
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(0) @binding(1) var texture1: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(2) var texture2: texture_storage_2d<rgba8unorm, write>;

@vertex fn path_tracer_vs(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4f {
  return vec4f(vertices[vid], 0.0, 1.0);
}



@fragment
fn path_tracer_fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    



    var frame = uniforms._pad;
    let x = (pos.x/f32(uniforms.width - 1) - 0.5)*2* f32(uniforms.width - 1) / f32(uniforms.height - 1);
    let y = -(pos.y/f32(uniforms.height - 1) - 0.5)*2;
    var uv = pos.xy / vec2(f32(uniforms.width), f32(uniforms.height));


    if(x < 0.01 && x > -0.01){
      if(y < 0.01 && y > -0.01){
        return vec4f(0.0,0.0,0.0,1.0);
      }
    }
    //spheres[0].position = uniforms.u;


    let camera_rotation = mat3x3(uniforms.u.xyz, uniforms.v.xyz, uniforms.w.xyz);
    let direction = camera_rotation * vec3(x, y, focus_distance);
    
    var thisray = Ray(uniforms.u.xyz, vec3(0,0,0));
    var colors = vec3<f32>(0, 0, 0);
    let coord:vec2<i32> = vec2<i32>(i32(pos.x),i32(pos.y));
    for(var j = 0; j < Samples; j++){
    var fovv =  vec3(x/uniforms.time,y/uniforms.time,-1);
    thisray = Ray(uniforms.u.xyz, uniforms.ww.xyz + fovv/abs(fovv));
    thisray.dir /= abs(thisray.dir);

    // Transparency shit

    var color = vec3<f32>(1 ,1 , 1);
    var transs = 1.0;
    for(var i = 0; i < MaxBounces; i++){
      let hit = RayBounce(thisray);
      if(!hit.hit){
        let k = 5*pow(dot(thisray.dir, sunDir)/(abs(sunDir)*abs(thisray.dir)), 51);
        //color *= vec3f(0.8, 0.8, 1.0);
        //color *= vec3f(joicy( 1 - 0.2*joicy(thisray.dir.y*1.0)),joicy(1 - 0.2*joicy(thisray.dir.y*1.0)), 1.0);
        let t = 0.5 * (normalize(thisray.dir).y + 1.);
        let sk = (1. - t) * vec3(1.) + t * vec3(0.3, 0.5, 1.);
        color*=sk;
        //color+=k;
        break;
      }
      if((hit.sphere.material.emission != vec3<f32>(0.0)).x||(hit.sphere.material.emission != vec3<f32>(0.0)).y||(hit.sphere.material.emission != vec3<f32>(0.0)).z){
        color += hit.sphere.material.emission;
        break;
      }

      var h = hit.sphere.material.color.xyz;

      let cos1 = dot(hit.normal, thisray.dir)/(abs(thisray.dir)*abs(hit.normal));
      let sin1 = sqrt(1 - cos1*cos1);
      let sin2 = sin1 / hit.sphere.material.refractiveIndex;
      var diss = thisray.dir - dot(thisray.dir, hit.normal)*hit.normal/abs(hit.normal);
      diss /= abs(diss);
      var p12 = diss*sin2 - hit.normal*sqrt(1 - sin2*sin2);
      p12/=abs(p12);
      var curr_pos = hit.pos;
      var kk = true;
      while(kk){
        var good_pos = dot(p12, -hit.normal)/abs(p12) * (-hit.normal);

        kk = false;
      }


      var firstRef = refract(thisray.dir,  hit.normal, hit.sphere.material.refractiveIndex);
      var transexes = transexit(hit.pos,p12);
      var fisray= Ray(transexes.exit_pos, transexes.exit_dir);
      var cols = vec3f(1);
      /*
      for(var i = 0; i < MaxTransBounces; i++){
        var random = random_direction(vec3<f32>(frame%112 + f32(j), x + f32(j)*2, y + f32(j)*6));
        let pit = RayBounce(fisray);
        if(!pit.hit){
          let k = 5*pow(dot(thisray.dir, sunDir)/(abs(sunDir)*abs(thisray.dir)), 51);
          //cols -= vec3f(1);
          //color*=k;
          //color *= vec3f(joicy( 1 - joicy(thisray.dir.y*0.5)),joicy(1 - joicy(thisray.dir.y*0.5)), 1.0);
           let t = 0.5 * (normalize(thisray.dir).y + 1.);
           let sk = (1. - t) * vec3(1.) + t * vec3(0.3, 0.5, 1.);
           cols*=sk;
          //color+=k;
          break;
        }
        random = (random + pit.normal)/abs(random + pit.normal);
        cols*=pit.sphere.material.color.xyz;
        var dis = random*(1-pit.sphere.material.reflections) - (2*dot(pit.normal, thisray.dir)*pit.normal - thisray.dir) * pit.sphere.material.reflections;
        fisray = Ray(hit.pos, dis);
      }
      */
      h = h *(1-hit.sphere.material.transparency) + cols * hit.sphere.material.transparency;
      color *= h;
      transs *= (1-hit.sphere.material.transparency);
      
      
      if(hit.sphere.material.transparency > 0.5){
        thisray = fisray;
        continue;
      }
      var random = random_direction(vec3<f32>(frame%10 + f32(j), x + f32(j)*2, y + f32(j)*3));

      if(dot(random, hit.normal) < 0){
        random *= -1;
      }
      random = (random + hit.normal)/abs(random + hit.normal);

      let cos_theta = max(0.0, dot(hit.normal, random));
      //color*=cos_theta*2;

      var dis = random*(1-hit.sphere.material.reflections) - (2*dot(hit.normal, thisray.dir)*hit.normal - thisray.dir) * hit.sphere.material.reflections;
      thisray = Ray(hit.pos, dis);
    }
    colors+=color;
  }

  colors/=f32(Samples);
  //colors*=16;
    frame = 5;
    var aa = (textureLoad(texture1, coord)*(frame-1) + (vec4(colors, 1.0)))/frame;
    textureStore(texture2, coord, aa);
    var divisor = 0.0;
    //  Anti - aliasing
    if(antiAliasing){
      aa*=level;

      for(var ind1:i32 = -times; ind1 <= times; ind1 += 1){
        for(var ind2:i32 = -times; ind2 <= times; ind2 += 1){
          if(coord.x + ind1 < 0 || coord.x + ind1 >= i32(uniforms.width) || coord.y + ind2 < 0 || coord.y + ind2 >= i32(uniforms.height)){
            continue;
          }
          aa += textureLoad(texture1, coord+vec2(ind1,ind2));
          divisor+=1.0;
        }
      }
      
      aa /= (level+divisor);
    }
    
    return vec4f(ACES(aa.xyz), 1.0);
}

fn ACES(x: vec3f) -> vec3f{
  let a = 2.51f;
  let b = 0.03f;
  let c = 2.43f;
  let d = 0.59f;
  let e = 0.14f;
  return pow(saturate((x*(a*x+b))/(x*(c*x+d)+e)), vec3f(1/2.2));
}

fn dothis(hello: vec3<f32>) -> vec3<f32>{
  return vec3<f32>(pow(kk(hello.x), 1/2.2), pow(kk(hello.y), 1/2.2), pow(kk(hello.z), 1/2.2));
}

fn kk(help: f32) -> f32{
  return help/ (1+help);
}


struct HitInfo{
  hit:bool,
  normal:vec3<f32>,
  pos:vec3<f32>,
  time:f32,
  sphere:Sphere
}

struct Material{
  color:vec4<f32>,
  reflections:f32,
  emission:vec3<f32>,
  transparency:f32,
  refractiveIndex:f32
}

fn RayBounce(ray:Ray) -> HitInfo{
  var getHit:HitInfo = HitInfo(false, vec3(1,1,1), vec3(0,0,0), 999999, spheres[0]);
  var dist:f32 = getHit.time;
  for(var i = 0; i < length; i++){
    var hit = Hit_sphere(spheres[i], ray);
    if(hit.hit){
      if(getHit.time > hit.time || !getHit.hit){getHit=hit;}
    }
  }
  return getHit;
}

fn Hit_sphere(sphere:Sphere, ray:Ray) -> HitInfo{
  let a:f32 = dot(sphere.position - ray.origin, ray.dir) / abs(ray.dir);
  let b:f32 = abs(sphere.position - ray.origin);
  let final1 = get_pos(ray.origin, ray.dir, sphere.position, sphere.radius);
  let final2 = gets_pos(ray.origin, ray.dir, sphere.position, sphere.radius);
  if(b*b - a*a < sphere.radius*sphere.radius && dot(ray.dir, final1-ray.origin) > 0.01){
    let normal = (final1-sphere.position)/abs(final1-sphere.position);
    return HitInfo(true, normal, final1, abs(final1 - ray.origin), sphere);
  }
  return HitInfo(false, vec3(1,1,1), vec3(0,0,0), 99999, sphere);
}

fn abs(vector:vec3f) -> f32{
  return pow(vector.x*vector.x + vector.y*vector.y + vector.z*vector.z, 0.5);
}

fn dot(vector1:vec3f, vector2:vec3f) -> f32{
  return vector1.x*vector2.x + vector1.y*vector2.y + vector1.z*vector2.z;
}

fn gets_pos(
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
        return vec3<f32>(9999.0, 9999.0, 9999.0);
    }
    let t = (-b + sqrt(discriminant)) / (2.0 * a);

    return origin + t * dir;
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
        return vec3<f32>(9999.0, 9999.0, 9999.0);
    }
    let t = (-b - sqrt(discriminant)) / (2.0 * a);

    return origin + t * dir;
}

struct transexit{
  exit_pos: vec3<f32>,
  exit_dir: vec3<f32>,
}

fn joicy(a:f32) -> f32{
  if(a < 0){
    return -a;
  }
  return a;
}

fn refract_dir(I: vec3<f32>, N: vec3<f32>, eta: f32) -> vec3<f32> {
    let cosi = clamp(-dot(I, N), -1.0, 1.0);
    let sint2 = eta * eta * (1.0 - cosi * cosi);

    if (sint2 > 1.0) {
        return reflect(I, N); // TIR
    }

    let cost = sqrt(1.0 - sint2);
    return normalize(eta * I + (eta * cosi - cost) * N);
}



fn refractions(
    entry_pos: vec3<f32>,
    ray_dir: vec3<f32>,
    normal: vec3<f32>,
    sphere: Sphere
) -> transexit {

    let I = normalize(ray_dir);
    var N = normalize(normal);

    var eta = 1.0 / sphere.material.refractiveIndex;

    // Ensure normal opposes ray
    if (dot(I, N) > 0.0) {
        N = -N;
        eta = 1.0 / eta;
    }

    // --- ENTER ---
    let T = refract_dir(I, N, eta);

    // --- EXIT INTERSECTION ---
    let L = entry_pos - sphere.position;
    let b = dot(T, L);
    let c = dot(L, L) - sphere.radius * sphere.radius;
    let h = b*b - c;

    // two intersections: t0 ~ 0, t1 > 0
    let t_exit = -b + sqrt(h);   // FAR HIT ONLY

    let exit_pos = entry_pos + T * t_exit;

    // --- EXIT NORMAL ---
    let N2 = normalize(exit_pos - sphere.position);

    // --- EXIT REFRACTION ---
    let T2 = refract_dir(T, N2, sphere.material.refractiveIndex);

    return transexit(exit_pos, T2);
}










fn refract_ray_through_sphere(
    entry_pos: vec3<f32>,
    normal: vec3<f32>,
    ray_dir: vec3<f32>,
    sphere_center: vec3<f32>,
    radius: f32,
    eta: f32, // refractive index of sphere (e.g. glass = 1.5)
) -> transexit {
    // Normalize inputs
    let I = normalize(ray_dir);
    let N = normalize(normal);

    // Compute refracted direction inside the sphere
    let eta_in = 1.0 / eta; // air to sphere
    let cosi = clamp(dot(-I, N), -1.0, 1.0);
    let sint2 = eta_in * eta_in * (1.0 - cosi * cosi);
    if sint2 > 1.0 {
        // Total internal reflection
        return transexit (entry_pos,reflect(I, N));
    }
    let cost = sqrt(1.0 - sint2);
    let refracted_in = eta_in * I + (eta_in * cosi - cost) * N;

    // Ray travels inside sphere: find exit point
    let oc = entry_pos - sphere_center;
    let b = dot(refracted_in, oc);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        // Shouldn't happen unless numerical error
        return transexit(
            entry_pos,
            refracted_in,
        );
    }
    let t_exit = -b + sqrt(discriminant);
    let exit_pos = entry_pos + t_exit * refracted_in;

    // Compute normal at exit
    let exit_normal = normalize(exit_pos - sphere_center);

    // Refract again: sphere to air
    let eta_out = eta;
    let cosi2 = clamp(dot(-refracted_in, exit_normal), -1.0, 1.0);
    let sint2_out = (1.0 / eta_out) * (1.0 / eta_out) * (1.0 - cosi2 * cosi2);
    if sint2_out > 1.0 {
        return transexit (
            exit_pos,
            reflect(refracted_in, exit_normal),
        );
    }
    let cost2 = sqrt(1.0 - sint2_out);
    let refracted_out = (1.0 / eta_out) * refracted_in + ((1.0 / eta_out) * cosi2 - cost2) * exit_normal;

    return transexit (
        exit_pos,
        normalize(refracted_out),
    );
}