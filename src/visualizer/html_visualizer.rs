use std::fs;
use std::io::Error;

use crate::visualizer::Visualizer;
use crate::fft_drawer;

pub struct HTMLVisualizer {
    file_name: String,
}

impl HTMLVisualizer {
    pub fn new(file_name: String) -> HTMLVisualizer {
        HTMLVisualizer{
            file_name: file_name,
        }
    }
}

impl Visualizer for HTMLVisualizer {
    fn render(&self, data: Vec<fft_drawer::DrawData>) -> bool {
        let fourier_json_data: String = data.iter()
            .map(|d| format!("{{\"s\": {:?}, \"r\": {:?}, \"a\": {:?}}},", d.frequency, d.radius, d.angle))
            .collect();
        // Strip the last comma
        let final_fourier_json_data: &str;
        if fourier_json_data.len() > 1 {
            final_fourier_json_data = &fourier_json_data[0..fourier_json_data.len() - 1];
        } else {
            final_fourier_json_data = &fourier_json_data;
        }
        let content = format!("<html>
<head>
    <title>Fourier Visualizer</title>
</head>
<canvas id=\"fourier_canvas\" width=\"800\" height=\"600\"></canvas>
<script>
/* FROM FourierFromSVG project */
let canvas = null; 
let context = null;
let time = 0;
const Point = class {{
    constructor(x, y) {{
        this.x = x;
        this.y = y;
    }}
    equals(point) {{
        return this.x === point.x && this.y === point.y;
    }}
    toString() {{
        return '(' + this.x + ', ' + this.y + ')';
    }}
}};

const FourierCircle = class {{
    constructor(speed, radius, initial_angle)
    {{
        this.radius = radius/2;
        this.speed = speed/20;
        this.initial_angle = initial_angle
    }}
    draw(ctx, at) 
    {{
        ctx.beginPath();
        //ctx.arc(at.x, at.y, this.radius, 0, Math.PI * 2, true);
        var x = at.x + this.radius * Math.cos(this.initial_angle + 2 * Math.PI * time * this.speed);
        var y = at.y + this.radius * Math.sin(this.initial_angle + 2 * Math.PI * time * this.speed);
        ctx.moveTo(at.x, at.y);
        ctx.lineTo(x, y);
        ctx.closePath();
        ctx.strokeStyle = 'rgba(202, 126, 86, 0.7)';
        ctx.lineWidth = 1; 
        ctx.stroke();
    }}

    nextCenter(at) 
    {{
        var x = at.x + this.radius * Math.cos(this.initial_angle + 2 * Math.PI * time * this.speed);
        var y = at.y + this.radius * Math.sin(this.initial_angle + 2 * Math.PI * time * this.speed);
        return new Point(x, y)
    }}
}};

let n;
let circles;
let animation_id = 0;
let center = new Point(150, 150);
let wave = [];

function init_fourier(canvas_elm, constants, count) {{
    canvas = canvas_elm;
    context = canvas.getContext('2d');
    if(animation_id !== 0)
        window.cancelAnimationFrame(animation_id);
    n = count;
    circles = [];
    wave = [];
    for (let i = 0; i < count; i++) {{
        let constant = constants[i];
        circles[i] = new FourierCircle(constant.s, constant.r, constant.a);
    }}
    animation_id = window.requestAnimationFrame(draw);
}}

function draw_wave(ctx) {{
    // ctx.beginPath();
    for (let i = 1; i < wave.length; i++) {{
        ctx.beginPath();
        ctx.moveTo(wave[i-1].x, wave[i-1].y);
        ctx.lineTo(wave[i].x, wave[i].y);
        ctx.closePath();

        // let c = Math.ceil(127.0 + 128.0*i/wave.length);
        let alpha = 1 - i*1.0/wave.length;
        
        ctx.strokeStyle = 'rgba(0, 0, 0, ' + alpha + ')';
        //ctx.strokeStyle = 'rgba(0, 0, 0, 1)';
        ctx.lineWidth = 1;
        ctx.stroke();
    }}
    // ctx.closePath();
    // ctx.strokeStyle = 'rgba(0, 0, 0, 1)';
    // ctx.stroke();
}}

function draw() {{
    context.clearRect(0,0, canvas.width, canvas.height);
    // let new_center = center;
    let new_center = circles[0].nextCenter(center);
    for(let i = 1; i < n; i++) {{
        circles[i].draw(context, new_center);
        new_center = circles[i].nextCenter(new_center);
    }}
    
    wave.unshift(new_center);
    draw_wave(context);

    animation_id = window.requestAnimationFrame(draw);

    time += 0.04;
    if(wave.length > 400) {{
        wave.pop();
    }}
}}
/* GEN */
window.onload = function() {{
    canvas = document.getElementById(\"fourier_canvas\");
    let data = JSON.parse(`[{}]`);
    init_fourier(canvas, data, {:?});
}};
</script>
</html>", final_fourier_json_data, data.len());

        let save_to_file = |file_name: &str| -> Result<(), Error> {
            fs::write(file_name, content)?;
            Ok(())
        };
        if let Err(_err) = save_to_file(&self.file_name) {
            return false;
        }

        true
    }
}
