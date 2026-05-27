// Mode switching
document.getElementById('modeFileBtn').addEventListener('click', () => {
    currentMode = 'svg';
    document.getElementById('svgControls').classList.remove('hidden');
    document.getElementById('drawingControls').classList.add('hidden');
    document.getElementById('visualizeSvgBtn').disabled = true;
    context.clearRect(0, 0, canvas.width, canvas.height);
    updateStatus('SVG File mode: Load an SVG file to begin');
});

document.getElementById('modeDrawBtn').addEventListener('click', () => {
    currentMode = 'draw';
    document.getElementById('svgControls').classList.add('hidden');
    document.getElementById('drawingControls').classList.remove('hidden');
    document.getElementById('visualizeBtn').disabled = drawingPoints.length < 3;
    context.clearRect(0, 0, canvas.width, canvas.height);
    redrawCanvas();
    updateStatus('Drawing mode: Draw on the canvas');
});

// Sample rate control
document.getElementById('sampleRate').addEventListener('input', (e) => {
    document.getElementById('sampleValue').textContent = e.target.value;
});

// Clear, Undo, Redo buttons
document.getElementById('clearBtn').addEventListener('click', clearCanvas);
document.getElementById('undoBtn').addEventListener('click', undo);
document.getElementById('redoBtn').addEventListener('click', redo);

// Visualize button
document.getElementById('visualizeBtn').addEventListener('click', () => {
    if (drawingPoints.length < 3) return;
    
    let svgPath = 'M ' + drawingPoints[0].x + ' ' + drawingPoints[0].y;
    for (let i = 1; i < drawingPoints.length; i++) {
        svgPath += ' L ' + drawingPoints[i].x + ' ' + drawingPoints[i].y;
    }
    svgPath += ' Z';
    
    const sampleRate = parseInt(document.getElementById('sampleRate').value);
    updateStatus('Computing Fourier Transform...');
    
    if (window.__TAURI__ && window.__TAURI__.core) {
        window.__TAURI__.core.invoke('process_drawing', {
            path: svgPath,
            numSample: sampleRate
        })
        .then((data) => {
            fullFourierData = data;
            document.getElementById('waveCount').max = data.length;
            initFourierVisualization();
            updateStatus('Visualizing with ' + data.length + ' components');
            
            // Show visualization controls
            document.getElementById('visualizeControls').classList.remove('hidden');
            document.getElementById('drawingControls').classList.add('hidden');
        })
        .catch((err) => {
            console.error('Error:', err);
            updateStatus('Error: ' + err);
        });
    } else {
        updateStatus('Tauri bridge not available');
    }
});

// Wave count control
document.getElementById('waveCount').addEventListener('input', (e) => {
    updateWaveCount(parseInt(e.target.value));
});

// Speed control
document.getElementById('speedControl').addEventListener('input', (e) => {
    speed_multiplier = parseFloat(e.target.value);
    document.getElementById('speedValue').textContent = speed_multiplier.toFixed(1);
});

// Timeline control
document.getElementById('timelineControl').addEventListener('input', (e) => {
    time = parseFloat(e.target.value) / 100 * 10;
});

// Zoom control
document.getElementById('zoomControl').addEventListener('input', (e) => {
    zoom = parseFloat(e.target.value);
    document.getElementById('zoomValue').textContent = zoom.toFixed(1);
});

// Color controls
document.getElementById('epicycleColor').addEventListener('input', (e) => {
    epicycleColor = e.target.value;
});

document.getElementById('traceColor').addEventListener('input', (e) => {
    traceColor = e.target.value;
});

// Visibility controls
document.getElementById('showCircles').addEventListener('change', (e) => {
    showCircles = e.target.checked;
});

document.getElementById('showTrace').addEventListener('change', (e) => {
    showTrace = e.target.checked;
});

// SVG file loading
document.getElementById('loadSvgBtn').addEventListener('click', async () => {
    if (window.__TAURI__ && window.__TAURI__.dialog) {
        try {
            const selected = await window.__TAURI__.dialog.open({
                multiple: false,
                filters: [{ name: 'SVG', extensions: ['svg'] }]
            });
            
            if (selected) {
                currentFilePath = selected;
                updateStatus('Parsing SVG file...');
                const result = await window.__TAURI__.core.invoke('parse_svg_file', {
                    filePath: selected
                });
                
                svgPaths = result.paths;
                selectedPathIndex = -1;
                selectedPathData = null;
                
                // Render SVG paths on canvas
                updateCanvasWithSvgPaths();
                
                document.getElementById('visualizeSvgBtn').disabled = true;
                updateStatus('Loaded ' + svgPaths.length + ' paths. Click on a path to select it.');
            }
        } catch (err) {
            console.error('Error loading SVG:', err);
            updateStatus('Error loading SVG: ' + err);
        }
    } else {
        updateStatus('File dialog not available');
    }
});

// Path selection is now handled by canvas click events in drawing.js

// SVG sample rate control
document.getElementById('sampleRateSvg').addEventListener('input', (e) => {
    document.getElementById('sampleValueSvg').textContent = e.target.value;
});

// Visualize SVG path
document.getElementById('visualizeSvgBtn').addEventListener('click', () => {
    if (!selectedPathData) return;
    
    const sampleRate = parseInt(document.getElementById('sampleRateSvg').value);
    updateStatus('Processing SVG path...');
    
    if (window.__TAURI__ && window.__TAURI__.core) {
        window.__TAURI__.core.invoke('process_svg_path', {
            pathData: selectedPathData,
            numSample: sampleRate
        })
        .then((data) => {
            fullFourierData = data;
            document.getElementById('waveCount').max = data.length;
            initFourierVisualization();
            updateStatus('Visualizing SVG path with ' + data.length + ' components');
            
            // Show visualization controls
            document.getElementById('visualizeControls').classList.remove('hidden');
            document.getElementById('svgControls').classList.add('hidden');
        })
        .catch((err) => {
            console.error('Error:', err);
            updateStatus('Error: ' + err);
        });
    } else {
        updateStatus('Tauri bridge not available');
    }
});

// Preset templates
document.getElementById('presetSelect').addEventListener('change', (e) => {
    document.getElementById('loadPresetBtn').disabled = !e.target.value;
});

document.getElementById('loadPresetBtn').addEventListener('click', () => {
    const preset = document.getElementById('presetSelect').value;
    if (preset) {
        loadPresetShape(preset);
        document.getElementById('presetSelect').value = '';
        document.getElementById('loadPresetBtn').disabled = true;
    }
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

// Fullscreen button
document.getElementById('fullscreenBtn').addEventListener('click', () => {
    if (document.fullscreenElement) {
        document.exitFullscreen();
    } else {
        document.documentElement.requestFullscreen();
    }
});

// New Drawing button
document.getElementById('newDrawBtn').addEventListener('click', () => {
    clearCanvas();
    document.getElementById('visualizeControls').classList.add('hidden');
    document.getElementById('drawingControls').classList.remove('hidden');
    document.getElementById('coefficientsPanel').classList.add('hidden');
    updateStatus('Ready to draw');
});
