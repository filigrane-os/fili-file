pragma Singleton
import QtQuick
// Stand-in for the Rust object: the Favorites are an ordered list, the other sections are fixed
QtObject {
    id: f
    property bool keyNav: false
    property var favs: ["/a", "/b", "/c", "/d"]
    property var calls: []
    signal placesChanged()
    function tr(s) { return s; }
    function places(color, bright) {
        const out = f.favs.map(p => ({ section: "Favorites", name: p.slice(1), path: p, dev: "", removable: false, indent: false, mono: "", monoBright: "", full: false }));
        out.push({ section: "Locations", name: "Computer", path: "/", dev: "", removable: false, indent: false, mono: "", monoBright: "", full: false });
        return JSON.stringify(out);
    }
    function moveFavoriteTo(path, index) {
        calls.push(["to", path, index]);
        const v = f.favs.slice(), i = v.indexOf(path);
        if (i < 0) return false;
        const j = Math.min(Math.max(index, 0), v.length - 1);
        if (i === j) return false;
        v.splice(i, 1); v.splice(j, 0, path);
        f.favs = v; f.placesChanged(); return true;
    }
    function moveFavorite(path, delta) { return moveFavoriteTo(path, f.favs.indexOf(path) + delta); }
    function addFavorite(p) {}
    function mountVolume(d) {}
    function unmountVolume(d) {}
}
