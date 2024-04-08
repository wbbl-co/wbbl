import { Kbd, Table, Heading, ScrollArea, Button, Em } from "@radix-ui/themes";

export default function KeybindingDialogContents() {
  return (
    <>
      <Heading style={{ paddingBottom: "var(--space-3)" }} as="h2">
        Keyboard Shortcuts
      </Heading>
      <ScrollArea
        type="hover"
        style={{
          paddingTop: "var(--space-3)",
          maxHeight: "min(400px, 75dvh)",
          overflowX: "hidden",
        }}
      >
        <Table.Root variant="ghost">
          <Table.Header>
            <Table.Row>
              <Table.ColumnHeaderCell>Description</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>
                Binding (Click to Change)
              </Table.ColumnHeaderCell>
            </Table.Row>
          </Table.Header>

          <Table.Body>
            <Table.Row>
              <Table.RowHeaderCell>Copy</Table.RowHeaderCell>
              <Table.Cell>
                <Button asChild onClick={console.log}>
                  <Kbd size={"3"}>Ctrl + C</Kbd>
                </Button>
              </Table.Cell>
            </Table.Row>
          </Table.Body>
        </Table.Root>
      </ScrollArea>
    </>
  );
}
