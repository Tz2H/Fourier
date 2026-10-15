import { Viewport } from './viewport.js';
import { samplePlayback } from './playback.js';

const $ = id => document.getElementById(id);
const canvas = $('canvas'), context = canvas.getContext('2d');
const view = new Viewport(canvas);
const descriptions = {
  simple: '略去细碎线条，让主体轮廓更干净。',
  balanced: '保留主体轮廓，也照顾眼睛与手臂的细节。',
  fine: '描绘更柔和的边界，也可能保留更多阴影纹理。',
};
let documentData = null, sourceFile = null, detail = 'balanced', previewUrl = null;
let playing = false, progress = 0, busy = false, showCircles = true, dirty = true;
let lastTime = performance.now(), lastPercent = -1, pixelRatio = 1;

function updatePlayback() {
  $('play-icon').setAttribute('d', playing ? 'M6 5h4v14H6ZM14 5h4v14h-4Z' : 'm9 5 11 7-11 7Z');
  $('play').setAttribute('aria-label', playing ? '暂停绘制' : progress >= 1 ? '重新播放' : '播放绘制');
  $('play-label').textContent = busy ? '正在准备线条…' : playing ? '正在逐笔绘制' : progress >= 1 ? '绘制完成' : '播放，看看圆如何作画';
  $('stage-status').textContent = busy ? '轮廓计算中' : playing ? '圆在旋转，线在生长' : progress >= 1 ? '绘制完成 · 自由探索细节' : '已暂停 · 可以缩放查看';
}
function setBusy(value) {
  busy = value;
  $('loading').hidden = !value;
  $('stage').setAttribute('aria-busy', String(value));
  $('drop-zone').classList.toggle('busy', value);
  $('drop-zone').setAttribute('aria-disabled', String(value));
  for (const element of [$('file'), $('sample'), ...document.querySelectorAll('[data-detail]')]) element.disabled = value;
  for (const id of ['play', 'restart', 'progress']) $(id).disabled = value || !documentData;
  updatePlayback();
}

async function loadImage(file = sourceFile, preset = detail) {
  if (busy) return;
  $('error').hidden = true;
  if (file && file.size > 32 * 1024 * 1024) {
    $('error').textContent = '图片不能超过 32 MB，请先缩小图片。';
    $('error').hidden = false;
    return;
  }
  const wasPlaying = playing;
  playing = false;
  setBusy(true);
  try {
    const endpoint = file ? '/api/image' : '/api/sample';
    const response = await fetch(`${endpoint}?detail=${preset}`, file ? { method: 'POST', headers: { 'Content-Type': 'application/octet-stream' }, body: file } : {});
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || '图片处理失败，请重试。');
    documentData = data;
    sourceFile = file;
    detail = preset;
    if (previewUrl) URL.revokeObjectURL(previewUrl);
    previewUrl = file ? URL.createObjectURL(file) : null;
    $('preview').src = previewUrl || '/sample.jpeg';
    $('preview').alt = `当前输入图片：${file ? file.name : '奶蛙示例'}`;
    $('filename').textContent = file ? file.name : '奶蛙 · 示例图片';
    $('dimensions').textContent = `${data.size[0]} × ${data.size[1]} px`;
    $('stats').textContent = `${data.strokeCount} 条独立笔画 / 约 ${Math.round(data.duration)} 秒`;
    $('preset-description').textContent = descriptions[preset];
    for (const button of document.querySelectorAll('[data-detail]')) button.setAttribute('aria-pressed', String(button.dataset.detail === preset));
    const notes = ['断点平滑抬笔，轮廓之间不留连接线。'];
    if (data.clipped) notes.push('原图边缘有裁切。');
    if (data.omitted) notes.push(`已省略 ${data.omitted} 条细碎边缘。`);
    $('drawing-note').textContent = notes.join(' ');
    progress = 0;
    lastPercent = -1;
    playing = !view.reducedMotion;
    view.fit(data.bounds);
    dirty = true;
  } catch (error) {
    $('error').textContent = error instanceof TypeError ? '无法连接本机服务，请确认 cargo run 仍在运行，然后重新选择图片或点击“使用示例”。' : error.message;
    $('error').hidden = false;
    playing = wasPlaying;
    if (!documentData) $('stats').textContent = '等待导入图片';
  } finally {
    setBusy(false);
  }
}

$('file').addEventListener('change', event => {
  const file = event.target.files[0];
  if (file) loadImage(file);
  event.target.value = '';
});
$('sample').addEventListener('click', () => loadImage(null));
for (const button of document.querySelectorAll('[data-detail]')) button.addEventListener('click', () => loadImage(sourceFile, button.dataset.detail));
$('drop-zone').addEventListener('keydown', event => {
  if (!busy && (event.key === 'Enter' || event.key === ' ')) { event.preventDefault(); $('file').click(); }
});
for (const name of ['dragenter', 'dragover']) $('drop-zone').addEventListener(name, event => {
  event.preventDefault();
  if (!busy) $('drop-zone').classList.add('dragover');
});
for (const name of ['dragleave', 'drop']) $('drop-zone').addEventListener(name, event => {
  event.preventDefault();
  $('drop-zone').classList.remove('dragover');
  if (name === 'drop' && !busy && event.dataTransfer.files[0]) loadImage(event.dataTransfer.files[0]);
});
// 防止文件掉在画布或空白处时，浏览器离开当前页面。
for (const name of ['dragover', 'drop']) window.addEventListener(name, event => { if ([...event.dataTransfer.types].includes('Files')) event.preventDefault(); });

function togglePlay() {
  if (!documentData || busy) return;
  if (progress >= 1) progress = 0;
  playing = !playing;
  dirty = true;
  updatePlayback();
}
$('play').addEventListener('click', togglePlay);
$('restart').addEventListener('click', () => { progress = 0; playing = true; dirty = true; updatePlayback(); });
$('progress').addEventListener('input', event => {
  progress = Number(event.target.value) / 1000;
  playing = false;
  dirty = true;
  updatePlayback();
});
$('circles').addEventListener('click', () => {
  showCircles = !showCircles;
  $('circles').setAttribute('aria-pressed', String(showCircles));
  dirty = true;
});
$('zoom-in').addEventListener('click', () => view.requestZoom(view.targetZoom * 1.25));
$('zoom-out').addEventListener('click', () => view.requestZoom(view.targetZoom / 1.25));
for (const id of ['fit', 'zoom-value']) $(id).addEventListener('click', () => view.fit());
canvas.addEventListener('keydown', event => { if (event.code === 'Space') { event.preventDefault(); togglePlay(); } });
document.addEventListener('visibilitychange', () => { lastTime = performance.now(); });

new ResizeObserver(() => {
  view.width = canvas.clientWidth;
  view.height = canvas.clientHeight;
  resizeBuffer();
  view.dirty = true;
}).observe(canvas);
function resizeBuffer() {
  pixelRatio = Math.min(window.devicePixelRatio || 1, 3);
  canvas.width = Math.round(view.width * pixelRatio);
  canvas.height = Math.round(view.height * pixelRatio);
  dirty = true;
}
function paintPath(points, last, color, width, transform, tip = null, close = false) {
  if (!points.length) return;
  const { scale, x, y } = transform;
  context.beginPath();
  context.moveTo(x + points[0][0] * scale, y - points[0][1] * scale);
  for (let i = 1; i <= last; i++) context.lineTo(x + points[i][0] * scale, y - points[i][1] * scale);
  if (tip) context.lineTo(x + tip[0] * scale, y - tip[1] * scale);
  if (close) context.closePath();
  context.strokeStyle = color;
  context.lineWidth = width;
  context.stroke();
}
function draw() {
  context.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
  context.clearRect(0, 0, view.width, view.height);
  if (!documentData) return;
  const data = documentData, transform = view.transform();
  const { scale, x, y } = transform;
  context.lineCap = 'round';
  context.lineJoin = 'round';
  const state = samplePlayback(data, progress);
  const active = data.strokes[state.index];
  for (let index = 0; index < data.strokes.length; index++) {
    const stroke = data.strokes[index];
    paintPath(stroke.reference, stroke.reference.length - 1, '#dfe2e8', 1, transform, null, stroke.closed);
    if (index < state.index || (index === state.index && state.penUp)) {
      paintPath(stroke.curve, stroke.curve.length - 1, '#36415b', 1.7, transform, null, stroke.closed);
    }
  }
  if (!state.penUp) {
    if (showCircles && state.circleAlpha > 0) {
      context.save();
      context.globalAlpha = state.circleAlpha;
      for (let index = 0; index < active.coefficients.length; index++) {
        const radius = active.coefficients[index][0];
        if (radius * scale < 0.7) continue;
        const start = state.chain[index], end = state.chain[index + 1];
        context.beginPath();
        context.arc(x + start[0] * scale, y - start[1] * scale, radius * scale, 0, 2 * Math.PI);
        context.strokeStyle = '#8295cb48';
        context.lineWidth = 1;
        context.stroke();
        paintPath([start, end], 1, '#8494c270', 1, transform);
      }
      context.restore();
    }
    if (state.position > 0) paintPath(active.curve, Math.floor(state.position), '#36415b', 1.7, transform, state.tip);
  }
  const tip = state.tip;
  context.save();
  context.globalAlpha = state.tipAlpha;
  const endX = x + tip[0] * scale, endY = y - tip[1] * scale;
  context.beginPath();
  context.arc(endX, endY, 8, 0, Math.PI * 2);
  context.fillStyle = '#dd82551c';
  context.fill();
  context.beginPath();
  context.arc(endX, endY, 3.1, 0, Math.PI * 2);
  context.fillStyle = '#dd8255';
  context.fill();
  context.restore();
}
function frame(now) {
  const delta = Math.min((now - lastTime) / 1000, 0.05);
  lastTime = now;
  if (pixelRatio !== Math.min(window.devicePixelRatio || 1, 3)) resizeBuffer();
  if (playing && documentData && !busy && !document.hidden) {
    progress = Math.min(1, progress + delta / documentData.duration);
    dirty = true;
    if (progress >= 1) { playing = false; updatePlayback(); }
  }
  view.update(delta);
  if (dirty || view.dirty) {
    draw();
    $('zoom-value').textContent = `${Math.round(view.zoom * 100)}%`;
    $('zoom-in').disabled = view.targetZoom >= 12;
    $('zoom-out').disabled = view.targetZoom <= 0.25;
    $('progress').value = Math.round(progress * 1000);
    const percent = Math.floor(progress * 100);
    if (lastPercent !== percent) { $('progress-label').textContent = `${percent}%`; lastPercent = percent; }
    dirty = view.dirty = false;
  }
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
loadImage(null);
