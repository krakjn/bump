(function () {
  var root = document.documentElement;
  var still = matchMedia("(prefers-reduced-motion: reduce)").matches;

  var toggle = document.querySelector(".theme-toggle");
  if (toggle) {
    toggle.addEventListener("click", function () {
      var next = root.dataset.theme === "dark" ? "light" : "dark";
      root.dataset.theme = next;
      localStorage.setItem("theme", next);
    });
  }

  if (window.hljs) {
    document.querySelectorAll(".frame pre code").forEach(function (block) {
      window.hljs.highlightElement(block);
    });
  }

  document.querySelectorAll(".frame pre").forEach(function (pre) {
    var button = document.createElement("button");
    button.type = "button";
    button.className = "copy";
    button.textContent = "Copy";
    button.addEventListener("click", function () {
      navigator.clipboard.writeText(pre.querySelector("code").innerText).then(function () {
        button.textContent = "Copied";
        button.classList.add("done");
        setTimeout(function () {
          button.textContent = "Copy";
          button.classList.remove("done");
        }, 1400);
      });
    });
    pre.appendChild(button);
  });

  var bar = document.querySelector(".progress");
  function progress() {
    var max = document.documentElement.scrollHeight - window.innerHeight;
    bar.style.transform = "scaleX(" + (max > 0 ? window.scrollY / max : 0) + ")";
  }
  window.addEventListener("scroll", progress, { passive: true });
  progress();

  var links = {};
  document.querySelectorAll(".toc a[href^='#']").forEach(function (a) {
    links[decodeURIComponent(a.getAttribute("href").slice(1))] = a;
  });
  var headings = document.querySelectorAll(".frame h2[id], .frame h3[id]");
  if (headings.length && "IntersectionObserver" in window) {
    var observer = new IntersectionObserver(
      function (entries) {
        entries.forEach(function (entry) {
          if (!entry.isIntersecting) return;
          Object.keys(links).forEach(function (id) {
            links[id].classList.toggle("active", id === entry.target.id);
          });
        });
      },
      { rootMargin: "0px 0px -70% 0px" }
    );
    headings.forEach(function (h) {
      observer.observe(h);
    });
  }

  // Repository widget facts. Cached per tab to stay under the API rate limit.
  var source = document.querySelector(".source");
  if (source) {
    var fill = function (facts) {
      if (facts.tag) source.querySelector("[data-release]").textContent = facts.tag;
      if (facts.stars != null) source.querySelector("[data-stars]").textContent = facts.stars;
      if (facts.forks != null) source.querySelector("[data-forks]").textContent = facts.forks;
    };
    var cached = sessionStorage.getItem("bump-source");
    if (cached) {
      fill(JSON.parse(cached));
    } else {
      var api = "https://api.github.com/repos/krakjn/bump";
      var json = function (r) { return r.ok ? r.json() : {}; };
      Promise.all([
        fetch(api).then(json),
        fetch(api + "/releases/latest").then(json)
      ]).then(function (res) {
        var facts = { stars: res[0].stargazers_count, forks: res[0].forks_count, tag: res[1].tag_name };
        fill(facts);
        sessionStorage.setItem("bump-source", JSON.stringify(facts));
      }).catch(function () {});
    }
  }

  // Point the hero button at the reader's build. URLs live on the .fitting cards.
  function detectPlatform() {
    var ua = navigator.userAgent;
    var os = /Windows/.test(ua) ? "windows"
      : /Macintosh|Mac OS X/.test(ua) ? "macos"
      : /Linux|X11/.test(ua) && !/Android/.test(ua) ? "linux"
      : null;
    if (!os) return Promise.resolve(null);
    var arch = /aarch64|arm64/i.test(ua) ? "arm64" : os === "macos" ? "arm64" : "amd64";
    var data = navigator.userAgentData;
    if (!data || !data.getHighEntropyValues) return Promise.resolve(os + "-" + arch);
    return data.getHighEntropyValues(["architecture"]).then(function (v) {
      if (v.architecture === "arm") arch = "arm64";
      if (v.architecture === "x86") arch = "amd64";
      return os + "-" + arch;
    }, function () {
      return os + "-" + arch;
    });
  }

  var cta = document.querySelector("[data-download]");
  detectPlatform().then(function (platform) {
    var card = platform && document.querySelector('.fitting[data-platform="' + platform + '"]');
    if (!card) return;
    card.classList.add("is-yours");
    if (!cta) return;
    cta.href = card.href;
    cta.setAttribute("download", card.getAttribute("download"));
    var name = card.querySelector(".fitting-os").textContent + " " + card.querySelector(".fitting-arch").textContent;
    cta.querySelector("[data-download-label]").textContent = "Download for " + name;
  });

  // Hero terminal: types each command, then rolls the changed digits like an odometer.
  var term = document.querySelector("[data-term]");
  if (!term) return;

  var steps = [
    { cmd: "bump print", ver: "v1.4.2" },
    { cmd: "bump patch", from: "1.4.2", to: "1.4.3" },
    { cmd: "bump minor", from: "1.4.3", to: "1.5.0" },
    { cmd: "bump phase rc", from: "1.5.0", to: "1.5.0-rc.1" }
  ];

  function esc(s) {
    return s.replace(/[&<>]/g, function (c) {
      return { "&": "&amp;", "<": "&lt;", ">": "&gt;" }[c];
    });
  }

  function odometer(from, to, animate) {
    if (to.indexOf(from) === 0 && to.length > from.length) {
      return esc(from) + '<span class="phase">' + esc(to.slice(from.length)) + "</span>";
    }
    var html = "";
    for (var i = 0; i < to.length; i++) {
      var a = from.charAt(i);
      var b = to.charAt(i);
      if (animate && a !== b && /\d/.test(a) && /\d/.test(b)) {
        html += '<span class="roll"><span><span>' + a + "</span><span>" + b + "</span></span></span>";
      } else {
        html += esc(b);
      }
    }
    return html;
  }

  function prompt(cmd) {
    return '<span class="ps">$</span> <span class="cmd">' + esc(cmd) + "</span>";
  }

  function output(step, animate) {
    if (step.ver) return '<span class="ver">' + esc(step.ver) + "</span>";
    return '<span class="out">' + esc(step.from) + '</span> <span class="arrow">→</span> <span class="ver">' +
      odometer(step.from, step.to, animate) + "</span>";
  }

  var caret = '<span class="caret"></span>';

  if (still) {
    term.innerHTML = steps.map(function (s) {
      return prompt(s.cmd) + "\n" + output(s, false);
    }).join("\n") + "\n" + prompt("") + caret;
    return;
  }

  var done = [];
  function render(current) {
    term.innerHTML = done.concat(current).join("\n");
  }

  function run(i) {
    if (i === steps.length) {
      setTimeout(function () {
        done = [];
        run(0);
      }, 3600);
      return;
    }
    var step = steps[i];
    var n = 0;
    (function type() {
      render(prompt(step.cmd.slice(0, n)) + caret);
      if (n++ < step.cmd.length) {
        setTimeout(type, 55 + Math.random() * 60);
        return;
      }
      setTimeout(function () {
        done.push(prompt(step.cmd));
        done.push(output(step, true));
        render(prompt("") + caret);
        setTimeout(function () {
          done[done.length - 1] = output(step, false);
          run(i + 1);
        }, 1100);
      }, 280);
    })();
  }

  render(prompt("") + caret);
  setTimeout(function () { run(0); }, 700);
})();
