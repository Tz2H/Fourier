// 视图只改变显示变换，不参与图像处理或傅里叶计算。
export class Viewport {
  constructor(canvas) {
    this.canvas = canvas;
    this.zoom = this.targetZoom = 1;
    this.pan = [0, 0];
    this.bounds = [[-160, -160], [160, 160]];
    this.width = this.height = 1;
    this.pointers = new Map();
    this.anchor = [0, 0];
    this.dirty = true;
    this.reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    let nativeGesture = false, gestureZoom = 1;
    // Safari 的触控板捏合使用 GestureEvent；其他浏览器通常使用 Ctrl + wheel。
    canvas.addEventListener('gesturestart', event => {
      event.preventDefault();
      nativeGesture = true;
      gestureZoom = this.zoom;
      this.targetZoom = this.zoom;
    }, { passive: false });
    canvas.addEventListener('gesturechange', event => {
      event.preventDefault();
      this.requestZoom(gestureZoom * event.scale, this.position(event));
    }, { passive: false });
    canvas.addEventListener('gestureend', event => {
      event.preventDefault();
      nativeGesture = false;
    }, { passive: false });
    canvas.addEventListener('wheel', event => {
      event.preventDefault();
      if (nativeGesture) return;
      const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? this.height : 1;
      const speed = event.ctrlKey ? 0.009 : 0.002;
      this.requestZoom(this.targetZoom * Math.exp(-event.deltaY * unit * speed), this.position(event));
    }, { passive: false });
    canvas.addEventListener('dblclick', () => this.fit());
    canvas.addEventListener('pointerdown', event => {
      if (event.pointerType === 'mouse' && event.button !== 0) return;
      this.targetZoom = this.zoom;
      canvas.focus({ preventScroll: true });
      canvas.setPointerCapture(event.pointerId);
      this.pointers.set(event.pointerId, this.position(event));
      canvas.classList.add('dragging');
    });
    canvas.addEventListener('pointermove', event => {
      if (!this.pointers.has(event.pointerId)) return;
      const before = [...this.pointers.values()];
      const previous = this.pointers.get(event.pointerId);
      const point = this.position(event);
      this.pointers.set(event.pointerId, point);
      if (this.pointers.size === 1) {
        this.pan[0] += point[0] - previous[0];
        this.pan[1] += point[1] - previous[1];
      } else if (this.pointers.size === 2) {
        const after = [...this.pointers.values()];
        const center = points => [(points[0][0] + points[1][0]) / 2, (points[0][1] + points[1][1]) / 2];
        const distance = points => Math.hypot(points[0][0] - points[1][0], points[0][1] - points[1][1]);
        const oldCenter = center(before), newCenter = center(after);
        this.pan[0] += newCenter[0] - oldCenter[0];
        this.pan[1] += newCenter[1] - oldCenter[1];
        if (distance(before) > 1) this.zoomAt(this.zoom * distance(after) / distance(before), newCenter);
        this.targetZoom = this.zoom;
      }
      this.dirty = true;
    });
    const release = event => {
      this.pointers.delete(event.pointerId);
      if (!this.pointers.size) canvas.classList.remove('dragging');
    };
    for (const name of ['pointerup', 'pointercancel', 'lostpointercapture']) canvas.addEventListener(name, release);
    canvas.addEventListener('keydown', event => {
      if (['+', '=', '-', '0', 'ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(event.key)) {
        event.preventDefault();
        if (event.key === '+' || event.key === '=') this.requestZoom(this.targetZoom * 1.25);
        if (event.key === '-') this.requestZoom(this.targetZoom / 1.25);
        if (event.key === '0') this.fit();
        if (event.key === 'ArrowLeft') this.pan[0] += 35;
        if (event.key === 'ArrowRight') this.pan[0] -= 35;
        if (event.key === 'ArrowUp') this.pan[1] += 35;
        if (event.key === 'ArrowDown') this.pan[1] -= 35;
        this.dirty = true;
      }
    });
  }
  position(event) {
    const rect = this.canvas.getBoundingClientRect();
    return [event.clientX - rect.left, event.clientY - rect.top];
  }
  center() { return [this.width / 2, this.height / 2 - 10]; }
  fit(bounds = this.bounds) {
    this.bounds = bounds;
    this.zoom = this.targetZoom = 1;
    this.pan = [0, 0];
    this.dirty = true;
  }
  requestZoom(value, anchor = this.center()) {
    this.targetZoom = Math.max(0.25, Math.min(12, value));
    this.anchor = anchor;
    if (this.reducedMotion) this.zoomAt(this.targetZoom, anchor);
    this.dirty = true;
  }
  zoomAt(value, anchor) {
    const next = Math.max(0.25, Math.min(12, value));
    const center = this.center(), ratio = next / this.zoom;
    this.pan = this.pan.map((offset, i) => anchor[i] - center[i] - (anchor[i] - center[i] - offset) * ratio);
    this.zoom = next;
    this.dirty = true;
  }
  update(delta) {
    if (this.zoom === this.targetZoom) return;
    const difference = this.targetZoom - this.zoom;
    this.zoomAt(Math.abs(difference) < 0.0001 ? this.targetZoom : this.zoom + difference * (1 - Math.exp(-delta * 18)), this.anchor);
  }
  transform() {
    const [min, max] = this.bounds;
    const base = Math.min(Math.max(1, this.width - 96) / Math.max(1, max[0] - min[0]), Math.max(1, this.height - 220) / Math.max(1, max[1] - min[1]));
    const scale = base * this.zoom, center = this.center();
    return { scale, x: center[0] + this.pan[0] - (min[0] + max[0]) / 2 * scale, y: center[1] + this.pan[1] + (min[1] + max[1]) / 2 * scale };
  }
}
