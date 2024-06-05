import {
  Avatar,
  Flex,
  ScrollArea,
  Table,
  TextField,
  Text,
  Button,
  IconButton,
  HoverCard,
  Box,
  Heading,
  Link,
  DataList,
  Badge,
  Code,
} from "@radix-ui/themes";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import ApplicationMenu from "../../components/ApplicationMenu";
import MicroSearchIcon from "../../components/icons/micro/MicroSearchIcon";
import { useQuery } from "@tanstack/react-query";
import Fuse from "fuse.js";
import { useMemo, useState } from "react";
import CoreLineNewFolder from "../../components/icons/core-line/CoreLineNewFolder";
import CoreLineRefresh from "../../components/icons/core-line/CoreLineRefresh";
import { UserAvatar, UserAvatarMore } from "../../components/UserAvatar";

export const Route = createLazyFileRoute("/app/")({
  component: Index,
});

async function getProjects(): Promise<{
  next_cursor: string | null;
  prev_cursor: string | null;
  results: { name: string; created_at: string }[];
}> {
  return {
    results: [
      {
        name: "frogject",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },
      {
        name: "frogject 22",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },

      {
        name: "frogject 3",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },

      {
        name: "frogject 4",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },

      {
        name: "frogject 5",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },

      {
        name: "frogject 6",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },

      {
        name: "frogject 7",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },
      {
        name: "frogject 8",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },

      {
        name: "frogject 9",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },

      {
        name: "frogject 10",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },

      {
        name: "frogject 11",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },

      {
        name: "frogject 12",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },

      {
        name: "frogject 13",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },

      {
        name: "frogject 14",
        created_at:
          "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)",
      },
    ],
    next_cursor: null,
    prev_cursor: null,
  };
}

function stringToHash(string: string): number {
  let hash = 0;

  if (string.length == 0) return hash;

  for (let i = 0; i < string.length; i++) {
    let char = string.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash;
  }

  return hash;
}

function Index() {
  const { isPending, error, data, refetch } = useQuery({
    queryKey: ["projectData"],
    queryFn: () => getProjects(),
  });
  const navigate = useNavigate();

  const index = useMemo(() => {
    return new Fuse(data?.results ?? [], { keys: ["name"] });
  }, [data?.results]);

  const [query, setQuery] = useState("");
  const items = useMemo(() => {
    if (query.length === 0) {
      return data?.results ?? [];
    }
    return index.search(query).map((x) => ({ ...x.item }));
  }, [query, index]);

  if (isPending) {
    return "Loading...";
  }

  if (error) {
    return "An error has occurred: " + error.message;
  }

  return (
    <div>
      <ApplicationMenu path={[]} />
      <Flex justify={"end"} p={"4"} pt={"6"} gap="3" width={"100%"}>
        <IconButton
          onClick={() => {
            refetch();
          }}
          color="gray"
          variant="soft"
          size={"3"}
        >
          <CoreLineRefresh />
        </IconButton>
        <Button size={"3"}>
          <CoreLineNewFolder /> New Project
        </Button>
        <TextField.Root
          placeholder="Search"
          size={"3"}
          onChange={(evt) => setQuery(evt.target.value)}
          style={{ width: "65ch" }}
        >
          <TextField.Slot>
            <MicroSearchIcon />
          </TextField.Slot>
        </TextField.Root>
      </Flex>
      <div
        style={{
          paddingTop: "2em",
          paddingLeft: "var(--space-3)",
          paddingRight: "var(--space-3)",
        }}
      >
        <ScrollArea
          size={"2"}
          scrollbars="vertical"
          style={{ maxHeight: "calc(90vh - 2em - var(--space-6))" }}
        >
          <Table.Root size={"3"} variant="ghost">
            <Table.Header style={{ position: "sticky", top: 0 }}>
              <Table.Row>
                <Table.ColumnHeaderCell>Project Name</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>Owners</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>Recent Viewers</Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>

            <Table.Body>
              {items.map((x) => {
                return (
                  <Table.Row
                    key={x.name}
                    role="button"
                    className="project-list-row"
                    onClick={() => {
                      navigate({ to: `/app/${x.name.replace(" ", "+")}` });
                    }}
                  >
                    <Table.RowHeaderCell>
                      <Text size={"4"}>{x.name}</Text>
                    </Table.RowHeaderCell>
                    <Table.Cell>
                      <UserAvatar
                        email="abc@example.com"
                        name="Abigail Thorne"
                        userId="abc"
                        role="admin"
                      />
                    </Table.Cell>
                    <Table.Cell>
                      <Flex>
                        <UserAvatar
                          email="abc@example.com"
                          name="Emily Tudor"
                          userId="def"
                          role="admin"
                        />
                        <UserAvatar
                          email="abc@example.com"
                          name="Jody Forster"
                          userId="ghi"
                          role="member"
                        />

                        <UserAvatarMore
                          users={[
                            {
                              name: "Geoff Reiner",
                              email: "geoff@gmail.com",
                              role: "member",
                              userId: "abc",
                            },
                            {
                              name: "Nancy Stuart",
                              email: "nstuart@google.com",
                              role: "member",
                              userId: "abc",
                            },
                          ]}
                        />
                      </Flex>
                    </Table.Cell>
                  </Table.Row>
                );
              })}
            </Table.Body>
          </Table.Root>
        </ScrollArea>
      </div>
    </div>
  );
}
