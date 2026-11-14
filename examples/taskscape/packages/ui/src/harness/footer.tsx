import { Button } from "../components/ui/button";
import { Footer } from "../components/ui/footer";

function FooterHarness({ density, tone, edge, drag }: any) {
  return (
    <Footer density={density} tone={tone} edge={edge} drag={drag}>
      <Button data-slot="cancel" variant="outline" label="Cancel" />
      <Button data-slot="accept" variant="primary" label="Save" />
    </Footer>
  );
}

export default { id: "ui.footer", Component: FooterHarness, events: [] };
