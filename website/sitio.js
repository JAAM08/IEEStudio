// Comportamiento común de ieestudio.com: navegación, versión publicada en GitHub,
// listas de versiones y cambios, y revelado de secciones. Sin dependencias.
(function () {
  "use strict";
  var GH = "https://api.github.com/repos/JAAM08/IEEStudio";

  // ---------------------------------------------------------------- navegación
  var nav = document.getElementById("nav");
  if (nav) {
    var onScroll = function () { nav.classList.toggle("scrolled", window.scrollY > 8); };
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
    var toggle = nav.querySelector(".menu");
    var cerrar = function () { nav.classList.remove("open"); toggle && toggle.setAttribute("aria-expanded", "false"); };
    if (toggle) {
      toggle.addEventListener("click", function () {
        var open = nav.classList.toggle("open");
        toggle.setAttribute("aria-expanded", open ? "true" : "false");
      });
      nav.querySelectorAll(".nav-links a").forEach(function (a) { a.addEventListener("click", cerrar); });
      document.addEventListener("keydown", function (e) { if (e.key === "Escape") cerrar(); });
    }

    // En la portada, el enlace de la sección visible queda marcado.
    var enlaces = nav.querySelectorAll("a[data-seccion]");
    var secciones = [];
    enlaces.forEach(function (a) {
      var s = document.getElementById(a.getAttribute("data-seccion"));
      if (s && location.pathname.replace(/index\.html$/, "").split("/").filter(Boolean).length === 0) secciones.push([s, a]);
    });
    if (secciones.length && "IntersectionObserver" in window) {
      var io = new IntersectionObserver(function (entradas) {
        entradas.forEach(function (e) {
          secciones.forEach(function (par) {
            if (par[0] === e.target) {
              if (e.isIntersecting) par[1].setAttribute("aria-current", "location");
              else par[1].removeAttribute("aria-current");
            }
          });
        });
      }, { rootMargin: "-45% 0px -50% 0px" });
      secciones.forEach(function (par) { io.observe(par[0]); });
    }
  }

  // ---------------------------------------------------------------- revelado
  var revela = document.querySelectorAll(".revela, .flujo");
  if (revela.length && "IntersectionObserver" in window) {
    var ro = new IntersectionObserver(function (entradas) {
      entradas.forEach(function (e) {
        if (e.isIntersecting) { e.target.classList.add("visto"); ro.unobserve(e.target); }
      });
    }, { rootMargin: "0px 0px -12% 0px" });
    revela.forEach(function (el) { ro.observe(el); });
  } else {
    revela.forEach(function (el) { el.classList.add("visto"); });
  }

  // ---------------------------------------------------------------- GitHub
  var fecha = function (iso) {
    return new Date(iso).toLocaleDateString("es-MX", { day: "numeric", month: "long", year: "numeric" });
  };
  // Las respuestas de GitHub se guardan 10 minutos para no gastar el límite de consultas.
  var consultar = function (url) {
    var clave = "gh:" + url, guardado = null;
    try { guardado = JSON.parse(sessionStorage.getItem(clave) || "null"); } catch (e) {}
    if (guardado && Date.now() - guardado.t < 600000) return Promise.resolve(guardado.d);
    return fetch(url, { headers: { Accept: "application/vnd.github+json" } })
      .then(function (r) { if (!r.ok) throw new Error(r.status); return r.json(); })
      .then(function (d) { try { sessionStorage.setItem(clave, JSON.stringify({ t: Date.now(), d: d })); } catch (e) {} return d; });
  };
  var texto = function (s) { var d = document.createElement("div"); d.textContent = s; return d.innerHTML; };

  // Los botones de descarga llevan al instalador de la versión más reciente.
  // Si la consulta falla, se quedan en la página de versiones.
  var listaVersiones = document.getElementById("versiones");
  if (document.querySelector("a.descargar, .version, .version-tag") || listaVersiones) {
    consultar(GH + "/releases?per_page=10")
      .then(function (releases) {
        releases = releases.filter(function (r) { return !r.draft; });
        if (listaVersiones) {
          listaVersiones.innerHTML = releases.length ? releases.slice(0, 4).map(function (r) {
            return '<li><a href="' + texto(r.html_url) + '">' + texto(r.name || r.tag_name) + '</a><span class="f">' + fecha(r.published_at) + "</span>" +
              '<span class="e">' + (r.prerelease ? "versión de prueba" : "versión estable") + "</span></li>";
          }).join("") : '<li class="cargando">Aún no hay versiones publicadas.</li>';
        }
        for (var i = 0; i < releases.length; i++) {
          var rel = releases[i], assets = rel.assets || [];
          for (var j = 0; j < assets.length; j++) {
            if (!/-instalador\.exe$/i.test(assets[j].name)) continue;
            var url = assets[j].browser_download_url;
            document.querySelectorAll("a.descargar").forEach(function (a) { a.href = url; });
            var mb = Math.round(assets[j].size / 1048576);
            var v = rel.tag_name.replace(/^v/, "");
            var tipo = rel.prerelease ? "versión de prueba" : "versión estable";
            document.querySelectorAll(".version").forEach(function (p) {
              p.innerHTML = "<strong>" + texto(v) + "</strong>, " + tipo + ", publicada el " + fecha(rel.published_at) +
                ". Instalador de Windows de " + mb + " MB." + (rel.prerelease ? " Revisa los resultados antes de usarlos en un proyecto." : "");
            });
            document.querySelectorAll(".version-tag").forEach(function (s) { s.textContent = v; });
            return;
          }
        }
      })
      .catch(function () {
        var c = listaVersiones && listaVersiones.querySelector(".cargando");
        if (c) c.firstChild.textContent = "No se pudo consultar GitHub. ";
      });
  }

  var listaCambios = document.getElementById("cambios");
  if (listaCambios) {
    consultar(GH + "/commits?sha=main&per_page=5")
      .then(function (commits) {
        listaCambios.innerHTML = commits.map(function (c) {
          var msg = (c.commit.message || "").split("\n")[0];
          var pr = /^Merge pull request #(\d+)/.exec(msg);
          var cuerpo = (c.commit.message || "").split("\n").filter(Boolean)[1];
          if (pr && cuerpo) msg = cuerpo + " (#" + pr[1] + ")";
          return '<li><a href="' + texto(c.html_url) + '">' + texto(msg) + '</a><span class="f">' + fecha(c.commit.committer.date) + "</span></li>";
        }).join("");
      })
      .catch(function () {
        var c = listaCambios.querySelector(".cargando");
        if (c) c.firstChild.textContent = "No se pudo consultar GitHub. ";
      });
  }
})();
