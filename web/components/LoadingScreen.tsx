import { Heading } from "@radix-ui/themes";

export default function LoadingScreen() {
  return (
    <div style={{ display: "flex", height: "100vh", width: "100vw", alignItems: "center", justifyItems: "center", background: "var(--background)", fontSize: "2rem", color: "white", fontStyle: "italic" }}>
      <Heading size={"8"} style={{ width: '100%', fontFamily: 'var(--brand-font-family)', fontWeight: 'normal' }} color="lime" align={'center'} as="h1">Loading...</Heading>
    </div>
  );
}
