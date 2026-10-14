// Línea bajo la navegación al desplazarse y menú en pantallas chicas.
(function () {
  var nav = document.getElementById("nav");
  if (!nav) return;
  var onScroll = function () { nav.classList.toggle("scrolled", window.scrollY > 8); };
  window.addEventListener("scroll", onScroll, { passive: true });
  onScroll();
  var toggle = nav.querySelector(".menu-toggle");
  if (toggle) {
    toggle.addEventListener("click", function () {
      var open = nav.classList.toggle("open");
      toggle.setAttribute("aria-expanded", open ? "true" : "false");
    });
  }
})();

// Los botones de descarga llevan al instalador de la versión más reciente
// publicada en GitHub. Si la consulta falla, llevan a la página de versiones.
(function () {
  if (!document.querySelector("a.descargar, .version")) return;
  fetch("https://api.github.com/repos/JAAM08/IEEStudio/releases?per_page=10", {
    headers: { Accept: "application/vnd.github+json" }
  })
    .then(function (r) { return r.ok ? r.json() : []; })
    .then(function (releases) {
      for (var i = 0; i < releases.length; i++) {
        var rel = releases[i];
        if (rel.draft) continue;
        var assets = rel.assets || [];
        for (var j = 0; j < assets.length; j++) {
          if (!/-instalador\.exe$/i.test(assets[j].name)) continue;
          var url = assets[j].browser_download_url;
          document.querySelectorAll("a.descargar").forEach(function (a) { a.href = url; });
          var mb = Math.round(assets[j].size / 1048576);
          var tipo = rel.prerelease ? "versión de prueba" : "versión estable";
          var fecha = new Date(rel.published_at).toLocaleDateString("es-MX", { day: "numeric", month: "long", year: "numeric" });
          document.querySelectorAll(".version").forEach(function (p) {
            p.innerHTML = "<strong>" + rel.tag_name.replace(/^v/, "") + "</strong>, " + tipo +
              ", publicada el " + fecha + ". Instalador de Windows de " + mb + " MB." +
              (rel.prerelease ? " Revisa los resultados antes de usarlos en un proyecto." : "");
          });
          document.querySelectorAll(".version-tag").forEach(function (s) { s.textContent = rel.tag_name.replace(/^v/, ""); });
          return;
        }
      }
    })
    .catch(function () {});
})();
