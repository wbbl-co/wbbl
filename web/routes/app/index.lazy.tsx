import { Avatar, Flex, ScrollArea, Table, TextField, Text } from '@radix-ui/themes'
import { createLazyFileRoute, useNavigate } from '@tanstack/react-router'
import ApplicationMenu from '../../components/ApplicationMenu'
import MicroSearchIcon from '../../components/icons/micro/MicroSearchIcon';
import {
    QueryClient,
    QueryClientProvider,
    useQuery,
} from '@tanstack/react-query'


export const Route = createLazyFileRoute("/app/")({
    component: Index,

})

async function getProjects(): Promise<{ next_cursor: string | null, prev_cursor: string | null, results: { name: string, warrant: { relation: string, }, is_implicit: boolean, created_at: string }[] }> {
    return {
        "results": [
            {
                "name": "frogject",
                "warrant": {
                    "relation": "owner",
                },
                "is_implicit": true,
                "created_at": "Mon Jun 03 2024 14:42:59 GMT+0000 (Coordinated Universal Time)"
            }
        ],
        "next_cursor": null,
        "prev_cursor": null
    };
}

function Index() {
    const { isPending, error, data } = useQuery({
        queryKey: ['projectData'],
        queryFn: () => getProjects()
        // fetch('https://api.github.com/repos/TanStack/query').then((res) =>
        //     res.json(),
        // ),
    });

    const navigate = useNavigate();

    if (isPending) {
        return 'Loading...';
    }

    if (error) {
        return 'An error has occurred: ' + error.message
    }



    return (
        <div>
            <ApplicationMenu path={[]} />
            <Flex justify={'end'} p={'4'} pt={'6'} width={'100%'}>
                <TextField.Root placeholder='Search' size={'3'} style={{ width: '100%', maxWidth: '70ch' }}>
                    <TextField.Slot><MicroSearchIcon /></TextField.Slot>
                </TextField.Root>
            </Flex>
            <div style={{ paddingTop: '2em', paddingLeft: 'var(--space-3)', paddingRight: 'var(--space-3)' }}>

                <ScrollArea>
                    <Table.Root size={'3'} variant='ghost'>
                        <Table.Header>
                            <Table.Row>
                                <Table.ColumnHeaderCell>Project Name</Table.ColumnHeaderCell>
                                <Table.ColumnHeaderCell>Owners</Table.ColumnHeaderCell>
                                <Table.ColumnHeaderCell>Viewers</Table.ColumnHeaderCell>
                            </Table.Row>
                        </Table.Header>

                        <Table.Body>
                            {data.results.map(x => {
                                return <Table.Row role='button' onClick={() => { navigate({ to: `/app/${encodeURIComponent(x.name)}` }) }} className='project-list-row'>
                                    <Table.RowHeaderCell><Text size={'3'}>{x.name}</Text></Table.RowHeaderCell>
                                    <Table.Cell><Avatar radius='full' variant='solid' fallback='D' /></Table.Cell>
                                    <Table.Cell><Flex>
                                        <Avatar style={{ marginLeft: '-0.4em', }} variant='solid' color='blue' radius='full' fallback='E' />
                                        <Avatar style={{ marginLeft: '-0.4em', }} variant='solid' color='green' radius='full' fallback='F' />
                                        <Avatar style={{ marginLeft: '-0.4em', }} variant='solid' color='red' radius='full' fallback='G' />
                                        <Avatar style={{ marginLeft: '-0.4em', }} variant='solid' color='gray' radius='full' fallback='+5' />
                                    </Flex>
                                    </Table.Cell>
                                </Table.Row>
                            })}

                        </Table.Body>
                    </Table.Root>
                </ScrollArea>
            </div>
        </div >
    )
}
