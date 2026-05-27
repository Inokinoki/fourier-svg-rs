// Initialize Fourier visualization
function initFourierVisualization() {
    fourierData = fullFourierData.slice(0, current_wave_count);
    circles = [];
    for (let i = 0; i < fourierData.length; i++) {
        const d = fourierData[i];
        circles.push(new FourierCircle(i, d.s, d.r, d.a));
    }
    wave = [];
    time = 0;
    
    // Fourier coefficients already contain position information (DC component)
    // Do not add center offset - it would double-count the position
    center = { x: 0, y: 0 };
    
    if (animation_id) {
        cancelAnimationFrame(animation_id);
    }
    animation_id = window.requestAnimationFrame(draw);
    updateCoefficientsList();
}

// Update coefficients list
function updateCoefficientsList() {
    // Currently just a placeholder - coefficients shown via wave count slider
}

// Update wave count
function updateWaveCount(newCount) {
    const max = fullFourierData ? fullFourierData.length : 201;
    current_wave_count = Math.max(1, Math.min(newCount, max));
    document.getElementById('waveCount').value = current_wave_count;
    document.getElementById('waveValue').textContent = current_wave_count;
    if (fullFourierData) {
        initFourierVisualization();
    }
}

// Draw wave trace
function drawWave(ctx) {
    for (let i = 1; i < wave.length; i++) {
        ctx.beginPath();
        ctx.moveTo(wave[i - 1].x, wave[i - 1].y);
        ctx.lineTo(wave[i].x, wave[i].y);
        const alpha = 1 - i / wave.length;
        ctx.strokeStyle = showTrace ? traceColor : 'transparent';
        ctx.globalAlpha = alpha;
        ctx.lineWidth = 2;
        ctx.stroke();
    }
    ctx.globalAlpha = 1;
}

// Main draw loop
function draw() {
    context.clearRect(0, 0, canvas.width, canvas.height);
    
    let new_center = circles[0] ? circles[0].nextCenter(center) : center;
    
    for (let i = 1; i < circles.length; i++) {
        if (showCircles) {
            circles[i].draw(context, new_center);
        }
        new_center = circles[i].nextCenter(new_center);
    }
    
    wave.unshift(new_center);
    drawWave(context);
    
    // Draw original path for comparison
    if (showOriginalPath && drawingPoints.length > 1) {
        context.beginPath();
        context.moveTo(drawingPoints[0].x, drawingPoints[0].y);
        for (let i = 1; i < drawingPoints.length; i++) {
            context.lineTo(drawingPoints[i].x, drawingPoints[i].y);
        }
        context.strokeStyle = 'rgba(255, 0, 0, 0.3)';
        context.lineWidth = 1;
        context.stroke();
    }
    
    if (!is_paused) {
        time += 0.001 * speed_multiplier;
    }
    
    if (wave.length > 400) {
        wave.pop();
    }
    
    animation_id = window.requestAnimationFrame(draw);
}
