import { Button } from "../components/ui/button";
import { Footer } from "../components/ui/footer";

function FooterHarness({ emit, ...props }: any) {
  // The footer emits nothing; the demo buttons carry their own events. Row sheets use taller buttons.
  void emit;
  const size = props.density === "row" ? "lg" : "md";
  return (
    <Footer {...props}>
      <Button data-slot="cancel" variant="outline" size={size} label="Cancel" />
      <Button data-slot="accept" variant="primary" size={size} label="Save" />
    </Footer>
  );
}

export default { id: "ui.footer", Component: FooterHarness, events: [] };
