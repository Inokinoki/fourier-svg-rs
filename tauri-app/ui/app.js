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

// Color customization
let epicycleColor = '#667eea';
let traceColor = '#333333';

// Visibility controls
let showCircles = true;
let showTrace = true;
let showCirclesOutline = false;

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
        this.speed = speed / 20;
        this.radius = radius / 2;
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

// Initialize on load
window.addEventListener('DOMContentLoaded', () => {
    canvas.width = 700;
    canvas.height = 600;
    center = { x: 350, y: 300 };
    redrawCanvas();
});
