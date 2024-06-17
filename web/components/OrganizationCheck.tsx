import { useSuspenseQuery } from "@tanstack/react-query";
import { PropsWithChildren } from "react";

export default function OrganizationCheck(props: PropsWithChildren<{ relation: string | string[], op?: 'AllOf' | 'AnyOf' }>) {
    const { data } = useSuspenseQuery({
        queryKey: ['organization', 'check', Array.isArray(props.relation) ? props.relation.join(',') : props.relation],
        queryFn: async () => {
            const x = await fetch('/api/introspect/organization/check', {
                method: 'POST',
                headers: { 'Content-Type': "application/json" },
                credentials: 'same-origin',
                body: JSON.stringify({
                    checks: Array.isArray(props.relation) ? props.relation : [props.relation],
                    op: props.op
                })
            });
            return x.status === 200 ? true : false;
        }
    });

    if (data === true) {
        return props.children;
    } else {
        return undefined;
    }
}