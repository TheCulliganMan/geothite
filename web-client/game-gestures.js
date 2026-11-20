// CSS touch-action owns the game's gestures. Safari also emits proprietary
// gesture events; cancel those only on gameplay, leaving menus and text usable.
export function mountGameGestures({ document, surfaces }) {
  const onGame = target => Boolean(target?.closest?.(surfaces));
  const prevent = event => {
    if (onGame(event.target) && event.cancelable) event.preventDefault();
  };
  const preventPinch = event => {
    if (event.touches?.length > 1 &&
        Array.from(event.touches).some(touch => onGame(touch.target)) && event.cancelable) {
      event.preventDefault();
    }
  };
  const options = { passive: false };
  for (const type of ['gesturestart', 'gesturechange', 'gestureend', 'dblclick']) {
    document.addEventListener(type, prevent, options);
  }
  for (const type of ['touchstart', 'touchmove']) {
    document.addEventListener(type, preventPinch, options);
  }
}
