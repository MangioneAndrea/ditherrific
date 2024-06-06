import init, { Image, transform, Options } from "../../pkg/ditherrific.js";

init().then(() => {
	Object.keys(Options)
		.filter((n) => Number.isNaN(+n))
		.forEach((n) =>
			document.querySelector("#options").add(new Option(n, n))
		);
});

document.querySelector("#src").addEventListener("click", () => {
	document.querySelector("input").click();
});
document.querySelector("input").addEventListener("change", () => {
	document.querySelector("#src").src = URL.createObjectURL(
		document.querySelector("input").files[0]
	);
});

document.querySelector("#btn").addEventListener("click", () => {
	const img = document.querySelector("#src");
	const cnv = document.querySelector("#res");

	const w = (cnv.width = img.naturalWidth);
	const h = (cnv.height = img.naturalHeight);

	const ctx = cnv.getContext("2d");
	ctx.drawImage(img, 0, 0);
	const imgData = ctx.getImageData(0, 0, w, h);

	const image = Image.from_rgb(w, h, imgData.data);
	const result = transform(
		image,
		Options[document.querySelector("#options").value]
	);

	const pixels = result.to_rgb();
	ctx.putImageData(new ImageData(pixels, w, h), 0, 0);
});
