// Export as PNG
document.getElementById('exportPngBtn').addEventListener('click', async () => {
    if (!fullFourierData) { updateStatus('No data to export'); return; }
    try {
        const dataUrl = canvas.toDataURL('image/png');
        const filePath = await tauriDialogSave({ defaultName: 'fourier_visualization.png', filters: [{ name: 'PNG', extensions: ['png'] }] });
        if (filePath) {
            await tauriInvoke('save_canvas_as_png', { dataUrl, filePath });
            updateStatus('PNG saved: ' + filePath);
        }
    } catch (err) {
        updateStatus('Error exporting PNG: ' + err);
    }
});

// Export as JSON
document.getElementById('exportJsonBtn').addEventListener('click', async () => {
    if (!fullFourierData) { updateStatus('No data to export'); return; }
    try {
        const filePath = await tauriDialogSave({ defaultName: 'fourier_data.json', filters: [{ name: 'JSON', extensions: ['json'] }] });
        if (filePath) {
            await tauriInvoke('export_fourier_data', { data: fullFourierData, filePath, numSamples: fullFourierData.length });
            updateStatus('JSON saved: ' + filePath);
        }
    } catch (err) {
        updateStatus('Error exporting JSON: ' + err);
    }
});

// Export as GIF
document.getElementById('exportGifBtn').addEventListener('click', async () => {
    if (!fullFourierData) { updateStatus('No data to export'); return; }
    try {
        const filePath = await tauriDialogSave({ defaultName: 'fourier_animation.gif', filters: [{ name: 'GIF', extensions: ['gif'] }] });
        if (filePath) {
            updateStatus('Generating GIF...');
            await tauriInvoke('export_as_gif', { data: fullFourierData, filePath, frames: parseInt(document.getElementById('gifFrames').value), duration: parseFloat(document.getElementById('gifDuration').value) });
            updateStatus('GIF saved: ' + filePath);
        }
    } catch (err) {
        updateStatus('Error exporting GIF: ' + err);
    }
});

// Export as HTML
document.getElementById('exportHtmlBtn').addEventListener('click', async () => {
    if (!fullFourierData) { updateStatus('No data to export'); return; }
    try {
        const filePath = await tauriDialogSave({ defaultName: 'fourier_visualization.html', filters: [{ name: 'HTML', extensions: ['html'] }] });
        if (filePath) {
            await tauriInvoke('export_as_html', { data: fullFourierData, filePath });
            updateStatus('HTML saved: ' + filePath);
        }
    } catch (err) {
        updateStatus('Error exporting HTML: ' + err);
    }
});
