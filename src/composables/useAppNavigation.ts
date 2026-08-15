import { ref } from 'vue';

export const APP_VIEWS = ['repos', 'activity', 'settings'] as const;
export type AppView = (typeof APP_VIEWS)[number];

export function useAppNavigation(initialView: AppView = 'repos') {
  const currentView = ref<AppView>(initialView);
  const touchStartX = ref(0);
  const touchStartY = ref(0);
  const touchCurrentX = ref(0);
  const touchCurrentY = ref(0);
  const isNavSwiping = ref(false);

  function handleTouchStart(event: TouchEvent, isLocked: boolean) {
    if (isLocked) return;

    touchStartX.value = event.touches[0].clientX;
    touchStartY.value = event.touches[0].clientY;
    touchCurrentX.value = touchStartX.value;
    touchCurrentY.value = touchStartY.value;
    isNavSwiping.value = false;
  }

  function handleTouchMove(event: TouchEvent, isLocked: boolean) {
    if (isLocked) return;

    touchCurrentX.value = event.touches[0].clientX;
    touchCurrentY.value = event.touches[0].clientY;

    const deltaX = touchCurrentX.value - touchStartX.value;
    const deltaY = touchCurrentY.value - touchStartY.value;

    if (!isNavSwiping.value && Math.abs(deltaX) > 40 && Math.abs(deltaX) > Math.abs(deltaY) * 2.5) {
      isNavSwiping.value = true;
    }
  }

  function handleTouchEnd(isLocked: boolean) {
    if (isLocked || !isNavSwiping.value) return;

    const deltaX = touchCurrentX.value - touchStartX.value;
    const swipeThreshold = 120;

    if (Math.abs(deltaX) > swipeThreshold) {
      const currentIndex = APP_VIEWS.indexOf(currentView.value);

      if (deltaX > 0 && currentIndex > 0) {
        currentView.value = APP_VIEWS[currentIndex - 1];
        if ('vibrate' in navigator) navigator.vibrate(10);
      }

      if (deltaX < 0 && currentIndex < APP_VIEWS.length - 1) {
        currentView.value = APP_VIEWS[currentIndex + 1];
        if ('vibrate' in navigator) navigator.vibrate(10);
      }
    }

    isNavSwiping.value = false;
  }

  return {
    currentView,
    handleTouchStart,
    handleTouchMove,
    handleTouchEnd,
  };
}
