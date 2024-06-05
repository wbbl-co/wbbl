import {
  Avatar,
  Badge,
  Box,
  DataList,
  Flex,
  Heading,
  HoverCard,
  Link,
  Text,
} from "@radix-ui/themes";

const colors = [
  "red",
  "yellow",
  "blue",
  "lime",
  "orange",
  "violet",
  "green",
] as const;

function uuidToColor(id: string) {
  let hash = 0;
  id.split("").forEach((char) => {
    hash = char.charCodeAt(0) + ((hash << 5) - hash);
  });

  return colors[Math.abs(hash) % colors.length];
}

export type UserProfile = {
  name: string;
  userId: string;
  email: string;
  role: string;
  lastSeen: number;
};

export function UserAvatarList(props: { users: UserProfile[], onClick?: (user: UserProfile) => void }) {
  const first = props.users.slice(0, 3);
  const rest = props.users.slice(3);
  return <Flex>
    {first.map(user => <UserAvatar key={user.userId} {...user} onClick={props.onClick ? () => { props.onClick!(user); } : undefined} />)}
    {rest.length > 1 ?
      <UserAvatarMore users={rest} />
      : (rest.length > 0 ? <UserAvatar key={rest[0].userId} {...rest[0]} onClick={props.onClick ? () => { props.onClick!(rest[0]); } : undefined} /> : undefined)
    }</Flex>;

}
export function UserAvatar(props: UserProfile & { onClick?: () => void }) {
  return (
    <HoverCard.Root>
      <HoverCard.Trigger
        onClick={(evt) => {
          evt.preventDefault();
          evt.stopPropagation();
        }}
      >
        <Link href={`#`}>
          <Avatar
            src={`/auth/api/avatars/${props.userId}.png`}
            style={{ marginLeft: "-0.4em" }}
            variant="solid"
            color={uuidToColor(props.userId)}
            radius="full"
            fallback={props.name[0]}
          />
        </Link>
      </HoverCard.Trigger>
      <HoverCard.Content
        maxWidth="300px"
        onClick={(evt) => {
          evt.preventDefault();
          evt.stopPropagation();
        }}
      >
        <Flex gap="4">
          <Avatar
            src={`/auth/api/avatars/${props.userId}.png`}
            style={{ marginLeft: "-0.4em" }}
            variant="solid"
            color={uuidToColor(props.userId)}
            radius="full"
            fallback={props.name[0]}
          />
          <Box>
            <Heading size="3" as="h3">
              {props.name}
            </Heading>
            <Text as="div" size="2" color="gray" mb="2">
              <Link href={`mailto:${props.email}`}>{props.email}</Link>
            </Text>
            <DataList.Root size={"1"}>
              <DataList.Item align="center">
                <DataList.Label minWidth="88px">Role</DataList.Label>
                <DataList.Value>
                  <Badge
                    color={props.role === "admin" ? "amber" : "green"}
                    variant="soft"
                    radius="full"
                    style={{ textTransform: "capitalize" }}
                  >
                    {props.role}
                  </Badge>
                </DataList.Value>
              </DataList.Item>
              <DataList.Item align="center">
                <DataList.Label minWidth="88px">Last Seen</DataList.Label>
                <DataList.Value>2 days ago</DataList.Value>
              </DataList.Item>
            </DataList.Root>
          </Box>
        </Flex>
      </HoverCard.Content>
    </HoverCard.Root>
  );
}

export function UserAvatarMore(props: { users: UserProfile[], onClick?: (user: UserProfile) => void }) {
  return (
    <HoverCard.Root>
      <HoverCard.Trigger
        onClick={(evt) => {
          evt.preventDefault();
          evt.stopPropagation();
        }}
      >
        <Link href={`#`}>
          <Avatar
            style={{ marginLeft: "-0.4em" }}
            variant="solid"
            color="gray"
            radius="full"
            fallback={`+${props.users.length}`}
          />
        </Link>
      </HoverCard.Trigger>
      <HoverCard.Content
        onClick={(evt) => {
          evt.preventDefault();
          evt.stopPropagation();
        }}
      >
        <Flex direction={"column"} gap={"1"}>
          {props.users.map((user) => (
            <Link size={"2"} key={user.userId} href={`mailto:${user.email}`}>
              {user.email}
            </Link>
          ))}
        </Flex>
      </HoverCard.Content>
    </HoverCard.Root>
  );
}
