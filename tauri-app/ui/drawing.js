// Preset shapes library (uses canvas center dynamically)
function getCanvasCenter() {
    return { x: canvas.width / 2, y: canvas.height / 2 };
}

const presetShapes = {
    circle: () => {
        const { x: cx, y: cy } = getCanvasCenter();
        const points = [];
        const r = 100;
        for (let i = 0; i <= 100; i++) {
            const angle = (i / 100) * 2 * Math.PI;
            points.push({ x: cx + r * Math.cos(angle), y: cy + r * Math.sin(angle) });
        }
        return points;
    },
    square: () => {
        const { x: cx, y: cy } = getCanvasCenter();
        const size = 200;
        const half = size / 2;
        return [
            { x: cx - half, y: cy - half },
            { x: cx + half, y: cy - half },
            { x: cx + half, y: cy + half },
            { x: cx - half, y: cy + half },
            { x: cx - half, y: cy - half }
        ];
    },
    triangle: () => {
        const { x: cx, y: cy } = getCanvasCenter();
        const r = 120;
        return [
            { x: cx, y: cy - r },
            { x: cx + r * Math.cos(Math.PI / 6), y: cy + r * Math.sin(Math.PI / 6) },
            { x: cx - r * Math.cos(Math.PI / 6), y: cy + r * Math.sin(Math.PI / 6) },
            { x: cx, y: cy - r }
        ];
    },
    star: () => {
        const { x: cx, y: cy } = getCanvasCenter();
        const points = [];
        const r = 100;
        for (let i = 0; i <= 10; i++) {
            const angle = (i / 10) * 2 * Math.PI - Math.PI / 2;
            const radius = i % 2 === 0 ? r : r / 2;
            points.push({ x: cx + radius * Math.cos(angle), y: cy + radius * Math.sin(angle) });
        }
        return points;
    },
    heart: () => {
        const { x: cx, y: cy } = getCanvasCenter();
        const points = [];
        for (let t = 0; t <= 2 * Math.PI; t += 0.05) {
            const x = 16 * Math.pow(Math.sin(t), 3);
            const y = -(13 * Math.cos(t) - 5 * Math.cos(2 * t) - 2 * Math.cos(3 * t) - Math.cos(4 * t));
            points.push({ x: cx + x * 6, y: cy + y * 6 });
        }
        return points;
    },
    spiral: () => {
        const { x: cx, y: cy } = getCanvasCenter();
        const points = [];
        for (let t = 0; t <= 6 * Math.PI; t += 0.1) {
            const r = t * 5;
            points.push({ x: cx + r * Math.cos(t), y: cy + r * Math.sin(t) });
        }
        return points;
    },
    infinity: () => {
        const { x: cx, y: cy } = getCanvasCenter();
        const points = [];
        const a = 100;
        for (let t = 0; t <= 2 * Math.PI; t += 0.05) {
            const x = a * Math.sin(t) / (1 + Math.pow(Math.cos(t), 2));
            const y = a * Math.sin(t) * Math.cos(t) / (1 + Math.pow(Math.cos(t), 2));
            points.push({ x: cx + x, y: cy + y });
        }
        return points;
    },
    sine: () => {
        const { x: cx, y: cy } = getCanvasCenter();
        const points = [];
        for (let x = -150; x <= 150; x += 3) {
            const y = 50 * Math.sin(x * 0.05);
            points.push({ x: cx + x, y: cy + y });
        }
        return points;
    }
};

// Load preset shape
function loadPresetShape(shapeName) {
    const shapeFunc = presetShapes[shapeName];
    if (shapeFunc) {
        drawingPoints = shapeFunc();
        redrawCanvas();
        updateStatus('Loaded preset: ' + shapeName);
    }
}

// Save state to undo stack
function saveToUndoStack() {
    undoStack.push([...drawingPoints]);
    redoStack = [];
    updateUndoRedoButtons();
}

// Undo
function undo() {
    if (undoStack.length > 0) {
        redoStack.push([...drawingPoints]);
        drawingPoints = undoStack.pop();
        redrawCanvas();
        updateUndoRedoButtons();
        updateStatus('Undo');
    }
}

// Redo
function redo() {
    if (redoStack.length > 0) {
        undoStack.push([...drawingPoints]);
        drawingPoints = redoStack.pop();
        redrawCanvas();
        updateUndoRedoButtons();
        updateStatus('Redo');
    }
}

// Update undo/redo buttons
function updateUndoRedoButtons() {
    document.getElementById('undoBtn').disabled = undoStack.length === 0;
    document.getElementById('redoBtn').disabled = redoStack.length === 0;
}

// Drawing handlers
let drawStartPos = null;
let previewPoints = [];

canvas.addEventListener('mousedown', (e) => {
    if (currentMode !== 'draw') return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    
    isDrawing = true;
    drawingStartTime = Date.now();
    drawingPointsWithTime = [];
    
    const tool = document.getElementById('drawingTool').value;
    if (tool === 'freehand') {
        saveToUndoStack();
        drawingPoints = [{ x, y }];
        drawingPointsWithTime.push({ x, y, t: 0 });
    } else {
        drawStartPos = { x, y };
    }
});

canvas.addEventListener('mousemove', (e) => {
    if (!isDrawing) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    
    const tool = document.getElementById('drawingTool').value;
    if (tool === 'freehand') {
        drawingPoints.push({ x, y });
        drawingPointsWithTime.push({ x, y, t: Date.now() - drawingStartTime });
        redrawCanvas();
    } else if (drawStartPos) {
        previewPoints = generateShapePoints(tool, drawStartPos, { x, y });
        redrawCanvas();
    }
});

canvas.addEventListener('mouseup', (e) => {
    if (!isDrawing) return;
    isDrawing = false;
    
    const tool = document.getElementById('drawingTool').value;
    if (tool !== 'freehand' && drawStartPos) {
        saveToUndoStack();
        drawingPoints = previewPoints;
        drawStartPos = null;
        previewPoints = [];
    }
    
    updateUI();
    redrawCanvas();
});

canvas.addEventListener('mouseleave', (e) => {
    if (isDrawing && document.getElementById('drawingTool').value === 'freehand') {
        isDrawing = false;
        updateUI();
    }
    // Clear hover state for SVG mode
    if (currentMode === 'svg') {
        hoveredPathIndex = -1;
        redrawCanvas();
    }
});

canvas.addEventListener('contextmenu', (e) => e.preventDefault());

// SVG path selection handlers
canvas.addEventListener('click', (e) => {
    if (currentMode !== 'svg' || svgPathElements.length === 0) return;
    
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    
    const pathIndex = findPathAtPoint(x, y);
    if (pathIndex >= 0) {
        selectedPathIndex = pathIndex;
        selectedPathData = svgPaths[pathIndex].d;
        
        // Update UI
        updateUI();
        updateStatus('Selected: ' + (svgPaths[pathIndex].id || 'Path ' + (pathIndex + 1)));
        
        redrawCanvas();
    }
});

canvas.addEventListener('mousemove', (e) => {
    if (currentMode !== 'svg' || svgPathElements.length === 0) return;
    
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    
    const pathIndex = findPathAtPoint(x, y);
    if (pathIndex !== hoveredPathIndex) {
        hoveredPathIndex = pathIndex;
        canvas.style.cursor = pathIndex >= 0 ? 'pointer' : 'default';
        redrawCanvas();
    }
});

// Generate shape points for tools
function generateShapePoints(tool, start, end) {
    const points = [];
    if (tool === 'line') {
        points.push(start);
        points.push(end);
    } else if (tool === 'rectangle') {
        points.push({ x: start.x, y: start.y });
        points.push({ x: end.x, y: start.y });
        points.push({ x: end.x, y: end.y });
        points.push({ x: start.x, y: end.y });
        points.push({ x: start.x, y: start.y });
    } else if (tool === 'ellipse') {
        const cx = (start.x + end.x) / 2;
        const cy = (start.y + end.y) / 2;
        const rx = Math.abs(end.x - start.x) / 2;
        const ry = Math.abs(end.y - start.y) / 2;
        for (let i = 0; i <= 50; i++) {
            const angle = (i / 50) * 2 * Math.PI;
            points.push({ x: cx + rx * Math.cos(angle), y: cy + ry * Math.sin(angle) });
        }
    }
    return points;
}

// Draw preview for shape tools
function drawPreview(points) {
    if (points.length < 2) return;
    context.beginPath();
    context.moveTo(points[0].x, points[0].y);
    for (let i = 1; i < points.length; i++) {
        context.lineTo(points[i].x, points[i].y);
    }
    context.strokeStyle = '#999';
    context.lineWidth = 1;
    context.setLineDash([5, 5]);
    context.stroke();
    context.setLineDash([]);
}

// Redraw canvas
function redrawCanvas() {
    context.clearRect(0, 0, canvas.width, canvas.height);
    
    // Draw hint when canvas is empty and not in SVG mode
    if (drawingPoints.length === 0 && !fourierData && svgPathElements.length === 0) {
        drawCanvasHint();
    }
    
    // Draw SVG paths if in SVG mode
    if (currentMode === 'svg' && svgPathElements.length > 0) {
        renderSvgPaths();
    }
    
    // Draw points
    if (drawingPoints.length > 1) {
        context.beginPath();
        context.moveTo(drawingPoints[0].x, drawingPoints[0].y);
        for (let i = 1; i < drawingPoints.length; i++) {
            context.lineTo(drawingPoints[i].x, drawingPoints[i].y);
        }
        context.strokeStyle = traceColor;
        context.lineWidth = 2;
        context.stroke();
    }
    
    // Draw preview
    if (previewPoints.length > 0) {
        drawPreview(previewPoints);
    }
}

// Draw canvas hint
function drawCanvasHint() {
    const cx = canvas.width / 2;
    const cy = canvas.height / 2;
    
    // Draw dashed border
    context.strokeStyle = '#ccc';
    context.lineWidth = 2;
    context.setLineDash([10, 10]);
    context.strokeRect(20, 20, canvas.width - 40, canvas.height - 40);
    context.setLineDash([]);
    
    // Draw crosshair
    context.strokeStyle = '#ddd';
    context.lineWidth = 1;
    context.beginPath();
    context.moveTo(cx - 30, cy);
    context.lineTo(cx + 30, cy);
    context.moveTo(cx, cy - 30);
    context.lineTo(cx, cy + 30);
    context.stroke();
    
    // Draw text
    context.fillStyle = '#999';
    context.font = '16px Segoe UI, sans-serif';
    context.textAlign = 'center';
    context.textBaseline = 'middle';
    context.fillText('Click and drag to draw a shape', cx, cy + 50);
    
    context.font = '12px Segoe UI, sans-serif';
    context.fillStyle = '#bbb';
    context.fillText('or select a template from the left panel', cx, cy + 75);
}


// Clear canvas
function clearCanvas() {
    drawingPoints = [];
    drawingPointsWithTime = [];
    fourierData = null;
    fullFourierData = null;
    undoStack = [];
    redoStack = [];
    svgPaths = [];
    selectedPathData = null;
    selectedPathIndex = -1;
    hoveredPathIndex = -1;
    updateUndoRedoButtons();
    redrawCanvas();
    updateUI();
    updateStatus('Canvas cleared');
}

// SVG path rendering and selection
let svgPathElements = []; // Parsed SVG path elements
let selectedPathIndex = -1;
let hoveredPathIndex = -1;

// Parse SVG path data into canvas path
function parseSvgPathToCanvas(pathData) {
    const path = new Path2D(pathData);
    return path;
}

// Render SVG paths on canvas
function renderSvgPaths() {
    if (svgPathElements.length === 0) return;
    
    svgPathElements.forEach((pathInfo, index) => {
        const path = pathInfo.path2d;
        
        // Draw path
        context.beginPath();
        context.strokeStyle = index === hoveredPathIndex ? '#667eea' : 
                             index === selectedPathIndex ? '#28a745' : '#999';
        context.lineWidth = index === hoveredPathIndex ? 3 : 
                           index === selectedPathIndex ? 3 : 1;
        context.stroke(path);
        
        // Draw path label
        if (pathInfo.bounds) {
            const labelX = pathInfo.bounds.x + pathInfo.bounds.width / 2;
            const labelY = pathInfo.bounds.y - 10;
            
            context.fillStyle = index === selectedPathIndex ? '#28a745' : '#666';
            context.font = '11px Segoe UI, sans-serif';
            context.textAlign = 'center';
            context.fillText(pathInfo.id, labelX, labelY);
        }
    });
}

// Calculate path bounds
function calculatePathBounds(pathData) {
    // Create a temporary canvas to calculate bounds
    const tempCanvas = document.createElement('canvas');
    const tempCtx = tempCanvas.getContext('2d');
    const path = new Path2D(pathData);
    
    // Get bounds by checking pixel data
    tempCanvas.width = canvas.width;
    tempCanvas.height = canvas.height;
    tempCtx.stroke(path);
    
    const imageData = tempCtx.getImageData(0, 0, tempCanvas.width, tempCanvas.height);
    let minX = tempCanvas.width, minY = tempCanvas.height, maxX = 0, maxY = 0;
    
    for (let y = 0; y < tempCanvas.height; y++) {
        for (let x = 0; x < tempCanvas.width; x++) {
            const i = (y * tempCanvas.width + x) * 4;
            if (imageData.data[i + 3] > 0) {
                minX = Math.min(minX, x);
                minY = Math.min(minY, y);
                maxX = Math.max(maxX, x);
                maxY = Math.max(maxY, y);
            }
        }
    }
    
    if (minX > maxX || minY > maxY) return null;
    
    return {
        x: minX,
        y: minY,
        width: maxX - minX,
        height: maxY - minY
    };
}

// Check if point is near a path
function isPointNearPath(x, y, pathData, threshold = 5) {
    const tempCanvas = document.createElement('canvas');
    const tempCtx = tempCanvas.getContext('2d');
    const path = new Path2D(pathData);
    
    tempCanvas.width = canvas.width;
    tempCanvas.height = canvas.height;
    tempCtx.lineWidth = threshold * 2;
    tempCtx.stroke(path);
    
    // Check if point is on the thick path
    const imageData = tempCtx.getImageData(
        Math.max(0, x - threshold),
        Math.max(0, y - threshold),
        threshold * 2,
        threshold * 2
    );
    
    for (let i = 0; i < imageData.data.length; i += 4) {
        if (imageData.data[i + 3] > 0) return true;
    }
    
    return false;
}

// Find path at point
function findPathAtPoint(x, y) {
    // Check paths in reverse order (top to bottom)
    for (let i = svgPathElements.length - 1; i >= 0; i--) {
        if (isPointNearPath(x, y, svgPathElements[i].d)) {
            return i;
        }
    }
    return -1;
}

// Update canvas with SVG paths
function updateCanvasWithSvgPaths() {
    svgPathElements = svgPaths.map((pathInfo, index) => {
        const path2d = parseSvgPathToCanvas(pathInfo.d);
        const bounds = calculatePathBounds(pathInfo.d);
        return {
            ...pathInfo,
            path2d,
            bounds
        };
    });
    
    redrawCanvas();
}
