/**
 * Fades out index.html's loading veil once the app has its first data, then removes it. The fade uses the Web
 * Animations API, which the reduced-motion clamp on CSS animations does not reach, so it fades in every motion
 * mode (as in the reference). Two frames first, so the loaded app has painted under the veil. Runs once.
 */
export function dismissBoot() {
  const boot = document.getElementById("boot");
  if (!boot || boot.dataset.dismissing) return;
  boot.dataset.dismissing = "1";
  boot.style.pointerEvents = "none";
  const remove = () => boot.remove();
  requestAnimationFrame(() =>
    requestAnimationFrame(() => {
      const fade = boot.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 300, easing: "cubic-bezier(0.2, 0, 0, 1)", fill: "forwards" });
      fade.onfinish = remove;
      setTimeout(remove, 600);
    }),
  );
}

// Never leave the veil up: if the first data has not arrived in this long, show what there is.
setTimeout(dismissBoot, 10_000);
