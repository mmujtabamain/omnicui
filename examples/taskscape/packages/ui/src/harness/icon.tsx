import { Icon, type IconProps } from "../components/ui/icon";

/* The tile is the enclosing element the contract hovers; it sets the parent colour that tone inherit takes.
   It sits in the middle of the page so the pointer at rest is not over it, and the media tones sit on a
   dark scrim, as they do in the lightbox. */
function IconHarness(props: IconProps) {
  const media = props.tone === "onMedia" || props.tone === "onMediaMuted";
  return (
    <div className="flex min-h-screen w-full items-center justify-center">
      <span
        data-slot="tile"
        className={`group/tile inline-flex items-center justify-center text-content-2 ${media ? "rounded-(--radius-control) bg-black/80 p-2" : ""}`}
      >
        <Icon {...props} />
      </span>
    </div>
  );
}

export default { id: "ui.icon", Component: IconHarness, events: [] };
