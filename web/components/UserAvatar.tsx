import {
  Avatar,
  Badge,
  Box,
  DataList,
  Flex,
  Heading,
  HoverCard,
  Link,
  Skeleton,
  Text,
} from "@radix-ui/themes";
import { useQueries, useQuery } from "@tanstack/react-query";
import { timeAgo } from "../utils/time-ago";

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

export function UserAvatarList(props: {
  users: string[];
  onClick?: (user: string) => void;
}) {
  const first = props.users.slice(0, 3);
  const rest = props.users.slice(3);
  return (
    <Flex>
      {first.map((user) => (
        <UserAvatar
          userId={user}
          key={user}
          onClick={
            props.onClick
              ? () => {
                  props.onClick!(user);
                }
              : undefined
          }
        />
      ))}
      {rest.length > 1 ? (
        <UserAvatarMore users={rest} />
      ) : rest.length > 0 ? (
        <UserAvatar
          key={rest[0]}
          userId={rest[0]}
          onClick={
            props.onClick
              ? () => {
                  props.onClick!(rest[0]);
                }
              : undefined
          }
        />
      ) : undefined}
    </Flex>
  );
}

async function getUser(user_id: string): Promise<{
  user_id: string;
  email: string;
  first_name?: string;
  last_name?: string;
  has_profile_picture: boolean;
  last_seen: number;
  role: string;
}> {
  return fetch(`/api/users/${user_id}`, {
    method: "GET",
    credentials: "same-origin",
  }).then((x) => x.json());
}

export function UserAvatar(props: { userId: string; onClick?: () => void }) {
  const query = useQuery({
    queryKey: ["user", props.userId],
    queryFn: () => getUser(props.userId),
  });
  return (
    <HoverCard.Root open={!query.isSuccess ? false : undefined}>
      <HoverCard.Trigger
        onClick={(evt) => {
          evt.preventDefault();
          evt.stopPropagation();
        }}
      >
          <Link href={`#`}>
            <Skeleton style={{ borderRadius: 'var(--radius-full)'}} loading={!query.isSuccess}>
              <Avatar
                src={
                  query.isSuccess && query.data.has_profile_picture
                    ? `/api/users/${props.userId}/profile_pic`
                    : undefined
                }
                style={{ marginLeft: "-0.4em" }}
                variant="solid"
                color={uuidToColor(props.userId)}
                radius="full"
                fallback={
                  query.isSuccess
                    ? `${query.data.first_name?.[0]}${query.data.last_name?.[0]}`
                    : "AC"
                }
              />
            </Skeleton>
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
            src={
              query.isSuccess && query.data.has_profile_picture
                ? `/api/users/${props.userId}/profile_pic`
                : undefined
            }
            style={{ marginLeft: "-0.4em" }}
            variant="solid"
            color={uuidToColor(props.userId)}
            radius="full"
            fallback={
              query.isSuccess
                ? `${query.data.first_name?.[0]} ${query.data.last_name?.[0]}`
                : "AC"
            }
          />
          <Box>
            <Heading size="3" as="h3">
              {`${query.data?.first_name} ${query.data?.last_name}`}
            </Heading>
            <Text as="div" size="2" color="gray" mb="2">
              <Link href={`mailto:${query.data?.email}`}>
                {query.data?.email}
              </Link>
            </Text>
            <DataList.Root size={"1"}>
              <DataList.Item align="center">
                <DataList.Label minWidth="88px">Role</DataList.Label>
                <DataList.Value>
                  <Badge
                    color={query.data?.role === "admin" ? "amber" : "green"}
                    variant="soft"
                    radius="full"
                    style={{ textTransform: "capitalize" }}
                  >
                    {query.data?.role}
                  </Badge>
                </DataList.Value>
              </DataList.Item>
              <DataList.Item align="center">
                <DataList.Label minWidth="88px">Last Seen</DataList.Label>
                <DataList.Value>{query.isSuccess? timeAgo(query.data.last_seen!) : ''}</DataList.Value>
              </DataList.Item>
            </DataList.Root>
          </Box>
        </Flex>
      </HoverCard.Content>
    </HoverCard.Root>
  );
}

export function UserAvatarMore(props: {
  users: string[];
  onClick?: (user: string) => void;
}) {
  const queries = useQueries({
    queries: props.users.map((x) => ({
      queryKey: ["user", x],
      queryFn: () => getUser(x),
    })),
  });
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
          {queries
            .map((x) => x.data)
            .filter((x) => !!x)
            .map((user) => (
              <Link
                size={"2"}
                key={user?.user_id}
                href={`mailto:${user?.email}`}
              >
                {user?.email}
              </Link>
            ))}
        </Flex>
      </HoverCard.Content>
    </HoverCard.Root>
  );
}
