/* Fades .landing-footer while real page content is scrolled underneath it, and back to fully
 * visible once the true bottom is reached. #landing-footer-sentinel sits at the end of
 * .landing-container's content, right where its 100px bottom padding begins, so the sentinel
 * entering the viewport is exactly the moment that padding already guarantees nothing sits
 * behind the footer - no separate pixel threshold to keep in sync with that CSS value.
 */
(function () {
  document.addEventListener('DOMContentLoaded', function () {
    var sentinel = document.getElementById('landing-footer-sentinel');
    var footer = document.querySelector('.landing-footer');
    if (!sentinel || !footer || !('IntersectionObserver' in window)) return;

    var observer = new IntersectionObserver(function (entries) {
      var entry = entries[entries.length - 1];
      footer.classList.toggle('landing-footer--faded', !entry.isIntersecting);
    });
    observer.observe(sentinel);
  });
})();
