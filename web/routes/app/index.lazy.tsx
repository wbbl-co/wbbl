import {
  Avatar,
  Flex,
  ScrollArea,
  Table,
  TextField,
  Text,
} from "@radix-ui/themes";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import ApplicationMenu from "../../components/ApplicationMenu";
import MicroSearchIcon from "../../components/icons/micro/MicroSearchIcon";
import { useQuery } from "@tanstack/react-query";
import Fuse from "fuse.js";
import { useMemo, useState } from "react";

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

function Index() {
  const { isPending, error, data } = useQuery({
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
      <Flex justify={"end"} p={"4"} pt={"6"} width={"100%"}>
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
                <Table.ColumnHeaderCell>Viewers</Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>

            <Table.Body>
              {items.map((x) => {
                return (
                  <Table.Row
                    key={x.name}
                    role="button"
                    onClick={() => {
                      navigate({
                        to: `/app/${x.name.replace(" ", "+")}`,
                      });
                    }}
                    className="project-list-row"
                  >
                    <Table.RowHeaderCell>
                      <Text size={"3"}>{x.name}</Text>
                    </Table.RowHeaderCell>
                    <Table.Cell>
                      <Avatar radius="full" variant="solid" fallback="D" />
                    </Table.Cell>
                    <Table.Cell>
                      <Flex>
                        <Avatar
                          style={{ marginLeft: "-0.4em" }}
                          variant="solid"
                          color="blue"
                          radius="full"
                          fallback="E"
                        />
                        <Avatar
                          style={{ marginLeft: "-0.4em" }}
                          variant="solid"
                          color="green"
                          radius="full"
                          fallback="F"
                        />
                        <Avatar
                          style={{ marginLeft: "-0.4em" }}
                          variant="solid"
                          color="red"
                          radius="full"
                          fallback="G"
                        />
                        <Avatar
                          style={{ marginLeft: "-0.4em" }}
                          variant="solid"
                          color="gray"
                          radius="full"
                          fallback="+5"
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
