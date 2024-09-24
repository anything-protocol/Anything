'use server'

import { revalidatePath } from "next/cache";
import { createClient } from "../supabase/server";
import { redirect } from "next/navigation";

export async function createInvitation(prevState: any, formData: FormData): Promise<{ token?: string, message?: string}> {
    "use server";

    const invitationType = formData.get("invitationType") as string;
    const accountId = formData.get("accountId") as string;
    const accountRole = formData.get("accountRole") as string;

    const supabase = createClient();

    const { data, error }: any = await supabase.rpc('create_invitation', 
         // @ts-ignore
         {
        account_id: accountId,
        invitation_type: invitationType,
        account_role: accountRole
    } as any);

    if (error) {
        return {
            message: error.message
        };
    }

    revalidatePath(`/dashboard/[accountSlug]/settings/members/page`);

    return {
        token: data.token as string
    }
};

export async function deleteInvitation(prevState: any, formData: FormData) {
    "use server";

    const invitationId = formData.get("invitationId") as string;
    const returnPath = formData.get("returnPath") as string;

    const supabase = createClient();

    const { error }: any = await supabase.rpc('delete_invitation',
         // @ts-ignore
        {
        invitation_id: invitationId
    } as any);

    if (error) {
        return {
            message: error.message
        };
    }
    redirect(returnPath);

};

export async function acceptInvitation(prevState: any, formData: FormData) {
    "use server";

    const token = formData.get("token") as string;

    const supabase = createClient();

    const { error, data }: any = await supabase.rpc('accept_invitation', 
         // @ts-ignore
        {
        lookup_invitation_token: token
    } as any);

    if (error) {
        return {
            message: error.message
        };
    }
    redirect(`/dashboard/${data.slug}`);

};