// === State-based panel management ===
function showPanel(panelId, visible) {
    const el = document.getElementById(panelId);
    if (el) el.classList.toggle('hidden', !visible);
}

function updateUI() {
    const isViz = (fullFourierData && fullFourierData.length > 0);

    if (currentMode === 'draw') {
        document.getElementById('modeDrawBtn').classList.add('active');
        document.getElementById('modeFileBtn').classList.remove('active');
        showPanel('drawPanel', !isViz);
        showPanel('svgPanel', false);
    } else {
        document.getElementById('modeFileBtn').classList.add('active');
        document.getElementById('modeDrawBtn').classList.remove('active');
        showPanel('drawPanel', false);
        showPanel('svgPanel', !isViz);
    }

    const hasDrawInput = currentMode === 'draw' && drawingPoints.length >= 3;
    const hasSvgInput = currentMode === 'svg' && selectedPathData;
    const showSampleAndAction = !isViz && (hasDrawInput || hasSvgInput);
    showPanel('samplePanel', showSampleAndAction);
    showPanel('visualizeAction', showSampleAndAction);
    showPanel('vizPanel', isViz);
}

// === Mode Switching ===
document.getElementById('modeDrawBtn').addEventListener('click', () => {
    currentMode = 'draw';
    context.clearRect(0, 0, canvas.width, canvas.height);
    fullFourierData = null;
    if (animation_id) { cancelAnimationFrame(animation_id); animation_id = null; }
    redrawCanvas();
    updateUI();
    updateStatus('Draw on the canvas or select a template');
});

document.getElementById('modeFileBtn').addEventListener('click', () => {
    currentMode = 'svg';
    fullFourierData = null;
    if (animation_id) { cancelAnimationFrame(animation_id); animation_id = null; }
    context.clearRect(0, 0, canvas.width, canvas.height);
    svgPathElements = [];
    selectedPathData = null;
    selectedPathIndex = -1;
    updateUI();
    updateStatus('Load an SVG file to begin');
});

// === Drawing Controls ===
document.getElementById('undoBtn').addEventListener('click', undo);
document.getElementById('redoBtn').addEventListener('click', redo);
document.getElementById('clearBtn').addEventListener('click', () => {
    clearCanvas();
    updateUI();
    updateStatus('Canvas cleared');
});

// === SVG Controls ===
document.getElementById('loadSvgBtn').addEventListener('click', async () => {
    try {
        const selected = await tauriDialogOpen({
            multiple: false,
            filters: [{ name: 'SVG', extensions: ['svg'] }]
        });

        if (selected) {
            currentFilePath = selected;
            updateStatus('Parsing SVG file...');
            const result = await tauriInvoke('parse_svg_file', { filePath: selected });

            svgPaths = result.paths;
            selectedPathIndex = -1;
            selectedPathData = null;

            updateCanvasWithSvgPaths();
            updateUI();
            updateStatus('Loaded ' + svgPaths.length + ' paths — click one to select');
        }
    } catch (err) {
        console.error('Error loading SVG:', err);
        updateStatus('Error: ' + err);
    }
});

// === Preset Templates ===
document.getElementById('presetSelect').addEventListener('change', (e) => {
    document.getElementById('loadPresetBtn').disabled = !e.target.value;
});

document.getElementById('loadPresetBtn').addEventListener('click', () => {
    const preset = document.getElementById('presetSelect').value;
    if (preset) {
        loadPresetShape(preset);
        document.getElementById('presetSelect').value = '';
        document.getElementById('loadPresetBtn').disabled = true;
        updateUI();
    }
});

// === Sample Rate ===
function isPowerOf2(n) { return n > 0 && (n & (n - 1)) === 0; }

function syncSampleRate(value) {
    const v = Math.max(256, Math.min(65536, value));
    document.getElementById('sampleRate').value = v;
    document.getElementById('sampleInput').value = v;
    document.getElementById('sampleValue').textContent = v;
    const hint = document.getElementById('sampleHint');
    if (hint) {
        hint.textContent = isPowerOf2(v)
            ? 'Higher rate = smoother curves, more Fourier components'
            : 'Warning: not a power of 2, FFT may be slower';
        hint.style.color = isPowerOf2(v) ? '#888' : '#dc3545';
    }
}

document.getElementById('sampleRate').addEventListener('input', (e) => {
    syncSampleRate(parseInt(e.target.value));
});

document.getElementById('sampleInput').addEventListener('change', (e) => {
    syncSampleRate(parseInt(e.target.value) || 8192);
});

document.getElementById('sampleMinus').addEventListener('click', () => {
    syncSampleRate(parseInt(document.getElementById('sampleInput').value) - 128);
});

document.getElementById('samplePlus').addEventListener('click', () => {
    syncSampleRate(parseInt(document.getElementById('sampleInput').value) + 128);
});

// === Visualize Button (handles both Draw and SVG modes) ===
document.getElementById('visualizeBtn').addEventListener('click', () => {
    const sampleRate = parseInt(document.getElementById('sampleRate').value);

    if (currentMode === 'draw') {
        if (drawingPoints.length < 3) {
            updateStatus('Need at least 3 points to visualize');
            return;
        }

        drawingBounds = calculateDrawingBounds(drawingPoints);
        center = drawingBounds ? { x: drawingBounds.centerX, y: drawingBounds.centerY } : { x: 0, y: 0 };

        let svgPath = 'M ' + drawingPoints[0].x + ' ' + drawingPoints[0].y;
        for (let i = 1; i < drawingPoints.length; i++) {
            svgPath += ' L ' + drawingPoints[i].x + ' ' + drawingPoints[i].y;
        }
        svgPath += ' Z';

        updateStatus('Computing Fourier Transform...');
        tauriInvoke('process_drawing', { path: svgPath, numSample: sampleRate })
            .then((data) => startVisualization(data))
            .catch((err) => { console.error(err); updateStatus('Error: ' + err); });

    } else if (currentMode === 'svg') {
        if (!selectedPathData) {
            updateStatus('Click on a path to select it first');
            return;
        }
        updateStatus('Processing SVG path...');
        tauriInvoke('process_svg_path', { pathData: selectedPathData, numSample: sampleRate })
            .then((data) => startVisualization(data))
            .catch((err) => { console.error(err); updateStatus('Error: ' + err); });
    }
});

function startVisualization(data) {
    fullFourierData = data;
    current_wave_count = Math.min(current_wave_count, data.length);
    document.getElementById('waveCount').max = data.length;
    document.getElementById('waveCount').value = current_wave_count;
    document.getElementById('waveInput').value = current_wave_count;
    document.getElementById('waveInput').max = data.length;
    document.getElementById('maxWaveValue').textContent = data.length;
    initFourierVisualization();
    updateUI();
    updateStatus('Visualizing with ' + current_wave_count + ' / ' + data.length + ' Fourier components');
}

// === Visualization Controls ===
function syncWaveCount(value) {
    const max = fullFourierData ? fullFourierData.length : 201;
    const clamped = Math.max(1, Math.min(value, max));
    document.getElementById('waveCount').value = clamped;
    document.getElementById('waveInput').value = clamped;
    document.getElementById('maxWaveValue').textContent = max;
    updateWaveCount(clamped);
}

document.getElementById('waveCount').addEventListener('input', (e) => {
    syncWaveCount(parseInt(e.target.value));
});

document.getElementById('waveInput').addEventListener('change', (e) => {
    syncWaveCount(parseInt(e.target.value) || 1);
});

document.getElementById('waveMinus').addEventListener('click', () => {
    const cur = parseInt(document.getElementById('waveInput').value) || 1;
    syncWaveCount(cur - 1);
});

document.getElementById('wavePlus').addEventListener('click', () => {
    const cur = parseInt(document.getElementById('waveInput').value) || 1;
    syncWaveCount(cur + 1);
});

document.getElementById('speedControl').addEventListener('input', (e) => {
    speed_multiplier = parseFloat(e.target.value);
    document.getElementById('speedValue').textContent = speed_multiplier.toFixed(1) + 'x';
});

document.getElementById('timelineControl').addEventListener('input', (e) => {
    time = parseFloat(e.target.value) / 100;
    wave = [];
});

document.getElementById('epicycleColor').addEventListener('input', (e) => {
    epicycleColor = e.target.value;
});

document.getElementById('traceColor').addEventListener('input', (e) => {
    traceColor = e.target.value;
});

// Display controls
document.getElementById('showCircles').addEventListener('change', (e) => {
    showCircles = e.target.checked;
});

document.getElementById('showTrace').addEventListener('change', (e) => {
    showTrace = e.target.checked;
});

document.getElementById('showOriginalPath').addEventListener('change', (e) => {
    showOriginalPath = e.target.checked;
});

// Pause/Resume
document.getElementById('pauseBtn').addEventListener('click', () => {
    is_paused = !is_paused;
    document.getElementById('pauseBtn').textContent = is_paused ? 'Resume' : 'Pause';
});

// Reset
document.getElementById('resetBtn').addEventListener('click', () => {
    time = 0;
    wave = [];
    is_paused = false;
    document.getElementById('pauseBtn').textContent = 'Pause';
});

// New Drawing
document.getElementById('newDrawBtn').addEventListener('click', () => {
    clearCanvas();
    fullFourierData = null;
    if (animation_id) { cancelAnimationFrame(animation_id); animation_id = null; }
    updateUI();
    updateStatus('Ready to draw');
});
