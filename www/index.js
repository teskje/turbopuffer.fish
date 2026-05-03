import init, { Game } from './pkg/turbopuffer_fish.js';

async function run() {
    await Promise.all([
        init(),
        document.fonts.load("24px JuliaMono"),
        document.fonts.load("bold 24px JuliaMono"),
    ]);

    const canvas = document.getElementById("canvas");
    const game = new Game(canvas);

    function canvasXY(event) {
        const rect = canvas.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;
        return [x, y];
    }

    canvas.addEventListener("mousedown", (event) => {
        const [x, y] = canvasXY(event);
        game.on_click(x, y);
    });

    canvas.addEventListener("mousemove", (event) => {
        const [x, y] = canvasXY(event);
        canvas.style.cursor = game.on_hover(x, y) ? "pointer" : "default";
    });

    canvas.addEventListener("touchstart", (event) => {
        event.preventDefault();
        for (const touch of event.changedTouches) {
            const [x, y] = canvasXY(touch);
            game.on_click(x, y);
        }
    });

    canvas.addEventListener("touchmove", (event) => {
        event.preventDefault();
        for (const touch of event.changedTouches) {
            const [x, y] = canvasXY(touch);
            game.on_hover(x, y);
        }
    });

    window.addEventListener("resize", () => {
        game.on_resize();
    });

    function loop(timestamp) {
        game.tick(timestamp);
        requestAnimationFrame(loop);
    }
    requestAnimationFrame(loop);
}

run();
