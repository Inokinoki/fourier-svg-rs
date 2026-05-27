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
    
    // Auto-center on drawing bounds
    if (drawingBounds) {
        center = { x: drawingBounds.centerX, y: drawingBounds.centerY };
    }
    
    if (animation_id) {
        cancelAnimationFrame(animation_id);
    }
    animation_id = window.requestAnimationFrame(draw);
    updateCoefficientsList();
    document.getElementById('coefficientsPanel').classList.remove('hidden');
}

// Update coefficients list
function updateCoefficientsList() {
    const list = document.getElementById('coefficientsList');
    if (!list) return;
    list.innerHTML = '';
    for (let i = 0; i < Math.min(10, fourierData.length); i++) {
        const d = fourierData[i];
        const item = document.createElement('div');
        item.className = 'coefficient-item';
        item.innerHTML = '<span>ω=' + d.s.toFixed(1) + '</span><span>r=' + (d.r * 2).toFixed(1) + '</span>';
        list.appendChild(item);
    }
    if (fourierData.length > 10) {
        const more = document.createElement('div');
        more.className = 'coefficient-item';
        more.textContent = '... (' + fourierData.length + ' total)';
        list.appendChild(more);
    }
}

// Update wave count
function updateWaveCount(newCount) {
    current_wave_count = Math.max(1, Math.min(newCount, fullFourierData ? fullFourierData.length : 201));
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
    
    // Draw grid if enabled
    if (document.getElementById('showGrid') && document.getElementById('showGrid').checked) {
        drawGrid();
    }
    
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
    if (drawingPoints.length > 1) {
        context.beginPath();
        context.moveTo(drawingPoints[0].x, drawingPoints[0].y);
        for (let i = 1; i < drawingPoints.length; i++) {
            context.lineTo(drawingPoints[i].x, drawingPoints[i].y);
        }
        context.strokeStyle = 'rgba(255, 0, 0, 0.3)';
        context.lineWidth = 1;
        context.stroke();
    }
    
    // Update timeline
    const timeline = document.getElementById('timelineControl');
    if (timeline) {
        timeline.value = (time % 10) / 10 * 100;
    }
    
    if (!is_paused) {
        time += 0.001 * speed_multiplier;
    }
    
    if (wave.length > 400) {
        wave.pop();
    }
    
    animation_id = window.requestAnimationFrame(draw);
}
