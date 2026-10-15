// 无状态时间轴：拖动进度、重播与逐帧播放共用同一套断点计算。
const clamp = value => Math.max(0, Math.min(1, value));
const ease = value => { const t = clamp(value); return t * t * t * (10 + t * (-15 + 6 * t)); };

export function samplePlayback(document, progress) {
  const seconds = clamp(progress) * document.duration;
  let index = document.strokes.findIndex(stroke => seconds < (stroke.travel?.end ?? stroke.end));
  if (index < 0) index = document.strokes.length - 1;
  const stroke = document.strokes[index];
  if (stroke.travel && seconds > stroke.end) {
    const phase = clamp((seconds - stroke.end) / (stroke.travel.end - stroke.end));
    const t = ease(phase), u = 1 - t;
    const [a, b, c, d] = stroke.travel.controls;
    const tip = [0, 1].map(axis => u ** 3 * a[axis] + 3 * u ** 2 * t * b[axis] + 3 * u * t ** 2 * c[axis] + t ** 3 * d[axis]);
    return { index, penUp: true, tip, tipAlpha: 1 - 0.55 * Math.sin(Math.PI * phase) ** 2 };
  }
  const phase = clamp((seconds - stroke.start) / (stroke.end - stroke.start));
  // 以重建曲线弧长重映射傅里叶时间，配合两端缓动避免断点突然加速。
  const distances = stroke.arcLengths;
  const target = distances.at(-1) * ease(phase);
  let low = 0, high = distances.length - 1;
  while (high - low > 1) {
    const middle = (low + high) >> 1;
    if (distances[middle] <= target) low = middle;
    else high = middle;
  }
  const span = distances[high] - distances[low];
  const fraction = span > 0 ? clamp((target - distances[low]) / span) : phase;
  const position = phase >= 1 ? distances.length - 1 : low + fraction;
  const time = position / (distances.length - 1) * (stroke.closed ? 1 : 0.5);
  let tip = [...stroke.origin];
  const chain = [tip];
  for (const [radius, frequency, angle] of stroke.coefficients) {
    const rotation = 2 * Math.PI * frequency * time + angle;
    tip = [tip[0] + radius * Math.cos(rotation), tip[1] + radius * Math.sin(rotation)];
    chain.push(tip);
  }
  return {
    index, penUp: false, tip, chain, position, tipAlpha: 1,
    circleAlpha: ease(Math.min(phase / 0.12, (1 - phase) / 0.12)),
  };
}
