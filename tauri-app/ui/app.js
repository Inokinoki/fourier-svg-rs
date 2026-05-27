// State
let isDrawing = false;
let drawingPoints = [];
let drawingPointsWithTime = [];
let fourierData = null;
let fullFourierData = null;
let currentMode = 'draw';
let svgPaths = [];
let selectedPathData = null;
let drawingStartTime = 0;
let currentFilePath = null;

// Undo/Redo state
let undoStack = [];
let redoStack = [];

// Animation state
let time = 0;
let animation_id = null;
let is_paused = false;
let speed_multiplier = 1.0;
let current_wave_count = 201;
let circles = [];
let wave = [];
let center = { x: 350, y: 300 };
let zoom = 1.0;
let panOffset = { x: 0, y: 0 };
let isPanning = false;
let lastPanPos = null;

// Drawing bounds for auto-centering
let drawingBounds = null;

// Color customization
let epicycleColor = '#667eea';
let traceColor = '#333333';

// Visibility controls
let showCircles = true;
let showTrace = true;
let showOriginalPath = true;

// Default parameters
let defaultSampleRate = 10240;
let defaultDuration = 10.0;

// Canvas references
const canvas = document.getElementById('fourier_canvas');
const context = canvas.getContext('2d');

// Point class
const Point = class {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }
    equals(point) {
        return this.x === point.x && this.y === point.y;
    }
    toString() {
        return '(' + this.x + ', ' + this.y + ')';
    }
};

// FourierCircle class
const FourierCircle = class {
    constructor(idx, speed, radius, initial_angle) {
        this.idx = idx;
        this.speed = speed;
        this.radius = radius;
        this.initial_angle = initial_angle;
    }
    draw(ctx, at) {
        ctx.beginPath();
        const x = at.x + this.radius * Math.cos(this.initial_angle + 2 * Math.PI * time * this.speed);
        const y = at.y + this.radius * Math.sin(this.initial_angle + 2 * Math.PI * time * this.speed);
        ctx.moveTo(at.x, at.y);
        ctx.lineTo(x, y);
        ctx.closePath();
        ctx.strokeStyle = epicycleColor;
        ctx.lineWidth = 1;
        ctx.stroke();
    }
    nextCenter(at) {
        const x = at.x + this.radius * Math.cos(this.initial_angle + 2 * Math.PI * time * this.speed);
        const y = at.y + this.radius * Math.sin(this.initial_angle + 2 * Math.PI * time * this.speed);
        return new Point(x, y);
    }
};

// Status update
function updateStatus(message) {
    const statusEl = document.getElementById('status');
    if (statusEl) statusEl.textContent = message;
}

// Tauri v2 invoke wrapper
async function tauriInvoke(command, args) {
    if (window.__TAURI_INTERNALS__) {
        return await window.__TAURI_INTERNALS__.invoke(command, args);
    } else {
        throw new Error('Tauri bridge not available');
    }
}

// Tauri v2 dialog wrapper
async function tauriDialogOpen(options) {
    if (window.__TAURI_INTERNALS__) {
        return await window.__TAURI_INTERNALS__.invoke('plugin:dialog|open', options);
    } else {
        throw new Error('Tauri dialog not available');
    }
}

async function tauriDialogSave(options) {
    if (window.__TAURI_INTERNALS__) {
        return await window.__TAURI_INTERNALS__.invoke('plugin:dialog|save', options);
    } else {
        throw new Error('Tauri dialog not available');
    }
}
// Resize canvas to fill container
function resizeCanvas() {
    const container = canvas.parentElement;
    const rect = container.getBoundingClientRect();
    const padding = 20; // container padding
    canvas.width = rect.width - padding;
    canvas.height = rect.height - padding;
    center = { x: canvas.width / 2, y: canvas.height / 2 };
    redrawCanvas();
}

// Initialize on load
window.addEventListener('DOMContentLoaded', () => {
    resizeCanvas();
    // Resize observer for responsive canvas
    const observer = new ResizeObserver(() => resizeCanvas());
    observer.observe(canvas.parentElement);
});

// Calculate drawing bounds
function calculateDrawingBounds(points) {
    if (points.length === 0) return null;
    
    let minX = Infinity, minY = Infinity;
    let maxX = -Infinity, maxY = -Infinity;
    
    for (const p of points) {
        minX = Math.min(minX, p.x);
        minY = Math.min(minY, p.y);
        maxX = Math.max(maxX, p.x);
        maxY = Math.max(maxY, p.y);
    }
    
    return {
        x: minX,
        y: minY,
        width: maxX - minX,
        height: maxY - minY,
        centerX: (minX + maxX) / 2,
        centerY: (minY + maxY) / 2
    };
}
