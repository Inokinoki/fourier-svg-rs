// Export as PNG
document.getElementById('exportPngBtn').addEventListener('click', async () => {
    if (!fullFourierData) {
        updateStatus('No data to export');
        return;
    }
    
    try {
        const dataUrl = canvas.toDataURL('image/png');
        if (window.__TAURI__ && window.__TAURI__.core) {
            const filePath = await window.__TAURI__.dialog.save({
                defaultPath: 'fourier_visualization.png',
                filters: [{ name: 'PNG', extensions: ['png'] }]
            });
            if (filePath) {
                await window.__TAURI__.core.invoke('save_canvas_as_png', {
                    dataUrl: dataUrl,
                    filePath: filePath
                });
                updateStatus('PNG exported to: ' + filePath);
            }
        }
    } catch (err) {
        console.error('Export PNG error:', err);
        updateStatus('Error exporting PNG: ' + err);
    }
});

// Export as JSON
document.getElementById('exportJsonBtn').addEventListener('click', async () => {
    if (!fullFourierData) {
        updateStatus('No data to export');
        return;
    }
    
    try {
        if (window.__TAURI__ && window.__TAURI__.core) {
            const filePath = await window.__TAURI__.dialog.save({
                defaultPath: 'fourier_data.json',
                filters: [{ name: 'JSON', extensions: ['json'] }]
            });
            if (filePath) {
                await window.__TAURI__.core.invoke('export_fourier_data', {
                    data: fullFourierData,
                    filePath: filePath,
                    numSamples: fullFourierData.length
                });
                updateStatus('JSON exported to: ' + filePath);
            }
        }
    } catch (err) {
        console.error('Export JSON error:', err);
        updateStatus('Error exporting JSON: ' + err);
    }
});

// Export as GIF
document.getElementById('exportGifBtn').addEventListener('click', async () => {
    if (!fullFourierData) {
        updateStatus('No data to export');
        return;
    }
    
    try {
        if (window.__TAURI__ && window.__TAURI__.core) {
            const filePath = await window.__TAURI__.dialog.save({
                defaultPath: 'fourier_animation.gif',
                filters: [{ name: 'GIF', extensions: ['gif'] }]
            });
            if (filePath) {
                updateStatus('Generating GIF...');
                await window.__TAURI__.core.invoke('export_as_gif', {
                    data: fullFourierData,
                    filePath: filePath,
                    frames: 100,
                    duration: 10.0
                });
                updateStatus('GIF exported to: ' + filePath);
            }
        }
    } catch (err) {
        console.error('Export GIF error:', err);
        updateStatus('Error exporting GIF: ' + err);
    }
});

// Export as HTML
document.getElementById('exportHtmlBtn').addEventListener('click', async () => {
    if (!fullFourierData) {
        updateStatus('No data to export');
        return;
    }
    
    try {
        if (window.__TAURI__ && window.__TAURI__.core) {
            const filePath = await window.__TAURI__.dialog.save({
                defaultPath: 'fourier_visualization.html',
                filters: [{ name: 'HTML', extensions: ['html'] }]
            });
            if (filePath) {
                await window.__TAURI__.core.invoke('export_as_html', {
                    data: fullFourierData,
                    filePath: filePath
                });
                updateStatus('HTML exported to: ' + filePath);
            }
        }
    } catch (err) {
        console.error('Export HTML error:', err);
        updateStatus('Error exporting HTML: ' + err);
    }
});
